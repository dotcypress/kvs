use byteorder::{BigEndian, ByteOrder};

use crate::config;
use crate::hole::Hole;
use crate::record::{hash_key, Record, RecordRef};
use crate::{Error, StoreAdapter};

const BUFFER_SIZE: usize = 256;
const HEADER_SIZE: usize = 7;
const REF_SIZE: usize = 6;
const HOLES_LEN: usize = config::CAPACITY / 4;
const MAX_KEY_SIZE: usize = BUFFER_SIZE - 6;
const INDEX_SIZE: usize = REF_SIZE * config::CAPACITY;
const METADATA_SIZE: usize = HEADER_SIZE + INDEX_SIZE;
const INDEX_LOAD_BATCH_SIZE: usize = BUFFER_SIZE / REF_SIZE * REF_SIZE;

pub struct KVStore<A: StoreAdapter> {
    adapter: A,
    buf: [u8; BUFFER_SIZE],
    holes: [Hole; HOLES_LEN],
    index: [RecordRef; config::CAPACITY],
}

impl<A, E> KVStore<A>
where
    A: StoreAdapter<Error = E>,
{
    pub fn open(adapter: A, allow_create: bool) -> Result<KVStore<A>, Error<E>> {
        if METADATA_SIZE as u32 > A::PAGE_SIZE * A::PAGES as u32 {
            return Err(Error::StoreOverflow);
        }

        let mut store = KVStore {
            adapter,
            buf: [0; BUFFER_SIZE],
            index: [RecordRef::default(); config::CAPACITY],
            holes: [Hole::default(); HOLES_LEN],
        };
        store.holes[0].from = METADATA_SIZE as u32;
        store.holes[0].to = A::PAGES as u32 * A::PAGE_SIZE;

        store
            .adapter
            .read(0, &mut store.buf[0..HEADER_SIZE as usize])
            .map_err(Error::AdapterError)?;

        if store.buf[0..4] != A::MAGIC {
            return if allow_create {
                store.format()?;
                Ok(store)
            } else {
                Err(Error::StoreNotFound)
            };
        }

        if store.buf[4] != config::VERSION {
            return Err(Error::InvalidVersion);
        }

        if BigEndian::read_u16(&store.buf[5..7]) as usize != config::CAPACITY {
            return Err(Error::InvalidCapacity);
        }

        let mut ref_idx = 0;
        let mut offset = HEADER_SIZE;
        while ref_idx < config::CAPACITY {
            let batch = usize::min(METADATA_SIZE - offset, INDEX_LOAD_BATCH_SIZE);
            let refs_per_batch = batch / REF_SIZE;

            store
                .adapter
                .read(offset as u32, &mut store.buf[..batch])
                .map_err(Error::AdapterError)?;

            for idx in 0..refs_per_batch {
                let ref_start = idx * REF_SIZE;
                store.index[ref_idx].data = BigEndian::read_u16(&store.buf[ref_start..]);

                if store.index[ref_idx].in_use() && !store.index[ref_idx].is_deleted() {
                    let rec_addr = BigEndian::read_u32(&store.buf[(ref_start + 2)..]);
                    store
                        .adapter
                        .read(rec_addr, &mut store.buf[..5])
                        .map_err(Error::AdapterError)?;

                    let val_cap = BigEndian::read_u16(&store.buf[..2]);
                    let key_len = store.buf[4] as u16;
                    store
                        .alloc(Some(rec_addr), (val_cap + key_len) as u32 + 5)
                        .ok_or(Error::StoreOverflow)?;
                }
                ref_idx += 1;
            }
            offset += batch;
        }
        Ok(store)
    }

    pub fn close(self) -> A {
        self.adapter
    }

    pub fn format(&mut self) -> Result<(), Error<E>> {
        self.index.iter_mut().for_each(|r| r.reset());
        self.holes.iter_mut().for_each(|h| h.reset());
        self.holes[0].from = METADATA_SIZE as u32;
        self.holes[0].to = A::PAGES as u32 * A::PAGE_SIZE;
        self.buf.iter_mut().for_each(|b| *b = 0xff);

        let mut offset = 0;
        while offset < METADATA_SIZE {
            let batch = usize::min(METADATA_SIZE - offset, BUFFER_SIZE);
            self.adapter
                .write_paged(offset as u32, &self.buf[..batch])
                .map_err(Error::AdapterError)?;
            offset += batch;
        }

        self.buf[0..4].copy_from_slice(&A::MAGIC);
        self.buf[4] = config::VERSION;
        BigEndian::write_u16(&mut self.buf[5..7], config::CAPACITY as u16);
        self.adapter
            .write_paged(0, &self.buf[..7])
            .map_err(Error::AdapterError)
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error<E>> {
        self.insert_with_capacity(key, val, None)
    }

    pub fn insert_with_capacity(
        &mut self,
        key: &[u8],
        val: &[u8],
        val_cap: Option<u16>,
    ) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);
        assert!(!val.is_empty() || val_cap.filter(|cap| *cap > 0).is_some());
        assert!(val_cap.filter(|cap| val.len() > *cap as usize).is_none());

        self.remove(key)?;

        let ref_idx = self.alloc_ref()?;
        let mut record = Record {
            ref_idx,
            addr: 0,
            key_len: key.len() as u8,
            value_len: val.len() as u16,
            value_cap: val_cap.unwrap_or(val.len() as u16),
        };

        if let Some(addr) = self.alloc(None, record.size()) {
            self.adapter
                .write_paged(addr + key.len() as u32 + 5, val)
                .map_err(Error::AdapterError)?;
            record.addr = addr;
            self.index[ref_idx].load_hash(hash_key(key));
            self.index[ref_idx].set_in_use();
            self.save_record_header(&record, Some(key))?;
            self.save_ref(ref_idx, addr)
        } else {
            Err(Error::StoreOverflow)
        }
    }

    pub fn append(&mut self, key: &[u8], patch: &[u8]) -> Result<(u16, u16), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);
        assert!(!patch.is_empty());

        let mut record = self
            .find_record(key)
            .and_then(|x| x.ok_or(Error::KeyNofFound))?;
        let offset = record.value_len;
        self.patch_value(&mut record, offset, patch)
    }

    pub fn patch(&mut self, key: &[u8], offset: u16, patch: &[u8]) -> Result<(u16, u16), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);
        assert!(!patch.is_empty());

        let mut record = self
            .find_record(key)
            .and_then(|x| x.ok_or(Error::KeyNofFound))?;

        self.patch_value(&mut record, offset, patch)
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<bool, Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);

        if let Some(rec) = self.find_record(key)? {
            self.free(rec.addr, rec.size());
            self.index[rec.ref_idx].set_deleted();
            self.save_ref(rec.ref_idx, u32::MAX)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn contains_key(&mut self, key: &[u8]) -> Result<bool, Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);
        self.find_record(key).map(|r| r.is_some())
    }

    pub fn load(&mut self, key: &[u8], buf: &mut [u8]) -> Result<(u16, u16, u16), Error<E>> {
        self.load_with_offset(key, 0, buf)
    }

    pub fn load_with_offset(
        &mut self,
        key: &[u8],
        offset: u16,
        buf: &mut [u8],
    ) -> Result<(u16, u16, u16), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE as usize);

        let rec = self
            .find_record(key)
            .and_then(|x| x.ok_or(Error::KeyNofFound))?;

        let read_len = u16::min(rec.value_len.saturating_sub(offset), buf.len() as u16);
        if offset + read_len > rec.value_len {
            return Err(Error::Overread);
        }

        let addr = rec.addr + rec.key_len as u32 + offset as u32 + 5;
        self.adapter
            .read(addr, &mut buf[..read_len as usize])
            .map_err(Error::AdapterError)?;

        Ok((read_len as u16, rec.value_len, rec.value_cap))
    }

    fn alloc_ref(&mut self) -> Result<usize, Error<E>> {
        if let Some((ref_idx, _)) = self
            .index
            .iter()
            .enumerate()
            .find(|(_, rec_ref)| !(**rec_ref).is_deleted() && !(**rec_ref).in_use())
        {
            Ok(ref_idx)
        } else if let Some((ref_idx, _)) = self
            .index
            .iter()
            .enumerate()
            .find(|(_, rec_ref)| (**rec_ref).is_deleted())
        {
            Ok(ref_idx)
        } else {
            Err(Error::StoreOverflow)
        }
    }

    fn alloc(&mut self, offset: Option<u32>, len: u32) -> Option<u32> {
        if let Some(offset) = offset {
            match self
                .holes
                .iter_mut()
                .find(|h| offset >= h.from && offset < h.to && h.size() - (offset - h.from) >= len)
            {
                Some(hole) if hole.from == offset => {
                    hole.from += len;
                    Some(offset)
                }
                Some(hole) => {
                    let hole_end = hole.to;
                    hole.to = offset;
                    if let Some(slot) = self.holes.iter_mut().find(|h| h.size() == 0) {
                        slot.from = offset + len;
                        slot.to = hole_end;
                    } else {
                        return None;
                    };
                    Some(offset)
                }
                _ => None,
            }
        } else {
            match self.holes.iter_mut().filter(|h| h.size() >= len).max() {
                Some(hole) => {
                    let start = hole.from;
                    hole.from += len;
                    Some(start)
                }
                _ => None,
            }
        }
    }

    fn free(&mut self, offset: u32, size: u32) {
        if let Some(hole) = self.holes.iter_mut().find(|h| h.to == offset) {
            hole.to += size;
        } else if let Some(hole) = self.holes.iter_mut().find(|h| h.from == offset + size) {
            hole.from = offset;
        } else if let Some(slot) = self.holes.iter_mut().find(|h| h.size() == 0) {
            slot.from = offset;
            slot.to = offset + size;
        }
    }

    fn find_record(&mut self, key: &[u8]) -> Result<Option<Record>, Error<E>> {
        assert!(!key.is_empty());
        let hash = crate::record::hash_key(key);
        for skip in 0..config::CAPACITY {
            if let Some((ref_idx, _)) = self
                .index
                .iter()
                .enumerate()
                .filter(|(_, rec_ref)| {
                    !rec_ref.is_deleted() && rec_ref.in_use() && rec_ref.get_hash() == hash
                })
                .nth(skip as usize)
            {
                let addr_ptr = HEADER_SIZE + ref_idx * REF_SIZE + 2;
                self.adapter
                    .read(addr_ptr as u32, &mut self.buf[..4])
                    .map_err(Error::AdapterError)?;

                let addr = BigEndian::read_u32(&self.buf[..4]);
                let ref_meta_size = usize::min(BUFFER_SIZE, 5 + key.len());
                self.adapter
                    .read(addr, &mut self.buf[0..ref_meta_size])
                    .map_err(Error::AdapterError)?;

                let key_len = self.buf[4];
                if key == &self.buf[5..(5 + key_len as usize)] {
                    let meta = Record {
                        ref_idx,
                        key_len,
                        addr,
                        value_cap: BigEndian::read_u16(&self.buf[..2]),
                        value_len: BigEndian::read_u16(&self.buf[2..4]),
                    };
                    return Ok(Some(meta));
                }
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }

    fn save_ref(&mut self, ref_idx: usize, addr: u32) -> Result<(), Error<E>> {
        assert!(ref_idx < self.index.len());
        BigEndian::write_u16(&mut self.buf[0..2], self.index[ref_idx].data);
        BigEndian::write_u32(&mut self.buf[2..6], addr);
        let offset = HEADER_SIZE + ref_idx * REF_SIZE;
        self.adapter
            .write_paged(offset as u32, &self.buf[..6])
            .map_err(Error::AdapterError)
    }

    fn save_record_header(&mut self, record: &Record, key: Option<&[u8]>) -> Result<(), Error<E>> {
        BigEndian::write_u16(&mut self.buf[0..2], record.value_cap);
        BigEndian::write_u16(&mut self.buf[2..4], record.value_len);

        let chunk = if let Some(key) = key {
            self.buf[4] = key.len() as u8;
            self.buf[5..(key.len() + 5)].copy_from_slice(key);
            key.len() + 5
        } else {
            4
        };

        self.adapter
            .write_paged(record.addr, &self.buf[..chunk])
            .map_err(Error::AdapterError)
    }

    fn patch_value(
        &mut self,
        record: &mut Record,
        offset: u16,
        patch: &[u8],
    ) -> Result<(u16, u16), Error<E>> {
        if offset > record.value_len {
            return Err(Error::InvalidPatchOffset);
        }

        let requested_len = offset + patch.len() as u16;
        if requested_len > record.value_cap {
            if self
                .alloc(
                    Some(record.addr + record.size()),
                    (requested_len - record.value_cap) as u32,
                )
                .is_none()
            {
                return Err(Error::StoreOverflow);
            }
            record.value_cap = requested_len;
        }

        let addr = record.addr + record.key_len as u32 + offset as u32 + 5;
        self.adapter
            .write_paged(addr, patch)
            .map_err(Error::AdapterError)?;

        if requested_len > record.value_len {
            record.value_len = requested_len;
            self.save_record_header(&record, None)?;
        }

        Ok((record.value_len, record.value_cap))
    }
}
