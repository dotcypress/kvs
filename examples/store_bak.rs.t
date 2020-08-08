use crate::alloc::Hole;
use crate::ptr::RecordRef;
use crate::{StoreAdapter, Error};

const VERSION: u8 = 1;
const REFS_LEN: usize = 256;
const HOLES_LEN: usize = 128;

const REF_SIZE: u16 = 4;
const MAX_PAGE_SIZE: usize = 128;
const MAX_KEY_SIZE: usize = 15;
const MAX_RECORD_SIZE: usize = 4095;

pub struct KVStore<A: StoreAdapter> {
    adapter: A,
    is_open: bool,
    index_pages: u16,
    holes: [Hole; HOLES_LEN],
    refs: [RecordRef; REFS_LEN],
}

impl<A, E> KVStore<A>
where
    A: StoreAdapter<Error = E>,
{
    pub fn new(adapter: A) -> KVStore<A> {
        let mut store = KVStore {
            adapter,
            is_open: false,
            index_pages: REFS_LEN as u16 * REF_SIZE / A::PAGE_SIZE,
            refs: [RecordRef::default(); REFS_LEN],
            holes: [Hole::default(); HOLES_LEN],
        };
        store.reset();
        store
    }

    pub fn create(&mut self) -> Result<(), Error<E>> {
        self.reset();
        let mut buf = [0; MAX_PAGE_SIZE];
        for ref_page_idx in 0..=self.index_pages {
            let offset = (ref_page_idx + 1) * A::PAGE_SIZE;
            self.adapter
                .write(offset, &buf[0..A::PAGE_SIZE as usize])
                .map_err(Error::AdapterError)?;
        }
        buf[0..4].copy_from_slice(&A::MAGIC);
        buf[4] = VERSION;
        buf[5] = self.index_pages as u8;
        self.adapter
            .write(0, &buf[0..A::PAGE_SIZE as usize])
            .map_err(Error::AdapterError)?;
        self.is_open = true;
        Ok(())
    }

    pub fn open(&mut self) -> Result<(), Error<E>> {
        self.reset();
        let mut page = [0; MAX_PAGE_SIZE];
        self.adapter
            .read(0, &mut page[0..A::PAGE_SIZE as usize])
            .map_err(Error::AdapterError)?;
        if page[0..4] != A::MAGIC {
            return Err(Error::StoreNotFound);
        }

        let ver = page[4];
        let index_pages = page[5] as u16;
        if ver != VERSION {
            return Err(Error::InvalidVersion);
        }
        if index_pages > self.index_pages {
            return Err(Error::IndexOverflow);
        }

        for page_idx in 0..self.index_pages {
            let offset = (page_idx + 1) * A::PAGE_SIZE;
            self.adapter
                .read(offset as u16, &mut page[0..A::PAGE_SIZE as usize])
                .map_err(Error::AdapterError)?;
            let refs_per_page = A::PAGE_SIZE / REF_SIZE;
            for ref_idx in 0..refs_per_page {
                let ref_offset = REF_SIZE * ref_idx;
                let mut rec_ref = RecordRef::deserialize(
                    &page[ref_offset as usize..(ref_offset + REF_SIZE) as usize],
                );
                let ref_idx = ref_idx + page_idx * refs_per_page;
                rec_ref.idx = ref_idx;
                if rec_ref.active() {
                    self.alloc(Some(rec_ref.page), rec_ref.pages(A::PAGE_SIZE))
                        .ok_or(Error::Overflow)?;
                }
                self.refs[ref_idx as usize] = rec_ref;
            }
        }

        self.is_open = true;
        Ok(())
    }

    pub fn contains_key(&mut self, key: &[u8]) -> Result<bool, Error<E>> {
        self.find_ref(key).map(|r| r.is_some())
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE);
        assert!(!val.is_empty() && val.len() + key.len() < MAX_RECORD_SIZE);
        if !self.is_open {
            return Err(Error::StoreClosed);
        }

        if self.contains_key(key)? {
            self.remove(key)?;
        }

        let mut rec_ref = if let Some(rec_ref) = self.refs.iter_mut().find(|r| !r.active()) {
            rec_ref.len = 1 + key.len() as u16 + val.len() as u16;
            *rec_ref
        } else {
            return Err(Error::Overflow);
        };

        if let Some(free_page) = self.alloc(None, rec_ref.pages(A::PAGE_SIZE)) {
            rec_ref.page = free_page;
        } else {
            return Err(Error::Overflow);
        }

        let mut buf = [0; MAX_PAGE_SIZE];
        let rec_end = u16::min(A::PAGE_SIZE, rec_ref.len) as usize;
        let val_start = key.len() + 1;
        let chunk_len = rec_end - val_start;

        buf[0] = key.len() as u8;
        buf[1..val_start].copy_from_slice(key);
        buf[val_start..rec_end].copy_from_slice(&val[..chunk_len]);
        self.adapter
            .write(rec_ref.page * A::PAGE_SIZE, &buf[0..rec_end])
            .map_err(Error::AdapterError)?;

        if rec_ref.len > A::PAGE_SIZE {
            let val_offset = val.len() - (rec_ref.len - A::PAGE_SIZE) as usize;
            for (idx, chunk) in val[val_offset..].chunks(A::PAGE_SIZE as usize).enumerate() {
                let page = rec_ref.page + idx as u16 + 1;
                self.adapter
                    .write(page * A::PAGE_SIZE, &chunk)
                    .map_err(Error::AdapterError)?
            }
        }

        rec_ref.hash = RecordRef::hash(key);
        self.refs[rec_ref.idx as usize] = rec_ref;
        self.save_ref(rec_ref)?;
        Ok(())
    }

    pub fn append(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE);
        assert!(!val.is_empty());
        if !self.is_open {
            return Err(Error::StoreClosed);
        }

        let mut rec_ref = if let Some(rec_ref) = self.find_ref(key)? {
            rec_ref
        } else {
            return Err(Error::KeyNofFound);
        };
        assert!(rec_ref.len as usize + val.len() < MAX_RECORD_SIZE);

        let initial_pages = rec_ref.pages(A::PAGE_SIZE);
        let initial_len = rec_ref.len;
        let last_page = rec_ref.page + rec_ref.pages(A::PAGE_SIZE);

        rec_ref.len += val.len() as u16;
        let new_pages = rec_ref.pages(A::PAGE_SIZE) - initial_pages;
        if new_pages > 0 && self.alloc(Some(last_page), new_pages).is_none() {
            return Err(Error::AppendFailed);
        }

        let used = initial_len % A::PAGE_SIZE;
        let first_chunk = if used > 0 {
            u16::min(val.len() as u16, A::PAGE_SIZE - used)
        } else {
            0
        };

        if first_chunk > 0 {
            let offset = (last_page - 1) * A::PAGE_SIZE + used;
            self.adapter
                .write(offset, &val[0..first_chunk as usize])
                .map_err(Error::AdapterError)?;
        }

        for (idx, chunk) in val[first_chunk as usize..]
            .chunks(A::PAGE_SIZE as usize)
            .enumerate()
        {
            let page = idx as u16 + last_page;
            self.adapter
                .write(page * A::PAGE_SIZE, &chunk)
                .map_err(Error::AdapterError)?
        }

        self.refs[rec_ref.idx as usize] = rec_ref;
        self.save_ref(rec_ref)?;
        Ok(())
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<(), Error<E>> {
        if !self.is_open {
            return Err(Error::StoreClosed);
        }
        if let Some(mut rec_ref) = self.find_ref(key)? {
            self.dealloc(rec_ref.page, rec_ref.pages(A::PAGE_SIZE));
            rec_ref.page = 0;
            rec_ref.len = 0;
            rec_ref.hash = 0;
            self.refs[rec_ref.idx as usize] = rec_ref;
            self.save_ref(rec_ref)?;
        }
        Ok(())
    }

    pub fn load_key(&mut self, hash: u16, buf: &mut [u8]) -> Result<usize, Error<E>> {
        assert!(!buf.is_empty());
        let rec_ref = if let Some(rec_ref) = self.refs.iter().find(|r| r.hash == hash) {
            rec_ref
        } else {
            return Err(Error::KeyNofFound);
        };
        self.adapter
            .read(rec_ref.page * A::PAGE_SIZE, &mut buf[0..1])
            .map_err(Error::AdapterError)?;
        let key_len = buf[0] as usize;
        self.adapter
            .read(rec_ref.page * A::PAGE_SIZE + 1, &mut buf[0..key_len])
            .map_err(Error::AdapterError)?;
        Ok(key_len)
    }

    pub fn load_val(
        &mut self,
        key: &[u8],
        offset: u16,
        buf: &mut [u8],
    ) -> Result<usize, Error<E>> {
        assert!(!buf.is_empty() && buf.len() < 256);
        if !self.is_open {
            return Err(Error::StoreClosed);
        }
        let rec_ref = if let Some(rec_ref) = self.find_ref(key)? {
            rec_ref
        } else {
            return Err(Error::KeyNofFound);
        };
        if offset >= (rec_ref.len - key.len() as u16 - 1) {
            return Ok(0);
        }
        let val_offset = offset as usize + key.len() + 1;
        let read_len = usize::min(
            rec_ref.len.saturating_sub(val_offset as u16) as usize,
            buf.len(),
        );
        let page = rec_ref.page as usize + (val_offset / A::PAGE_SIZE as usize);
        let val_offset = val_offset % A::PAGE_SIZE as usize;
        let offset = page * A::PAGE_SIZE as usize + val_offset;
        self.adapter
            .read(offset as u16, &mut buf[0..read_len])
            .map_err(Error::AdapterError)?;
        Ok(read_len)
    }

    fn alloc(&mut self, begin: Option<u16>, pages: u16) -> Option<u16> {
        assert!(pages > 0);
        if let Some(begin) = begin {
            assert!(begin > self.index_pages);
            match self
                .holes
                .iter_mut()
                .find(|h| begin >= h.from && begin < h.to && h.size() - (begin - h.from) >= pages)
            {
                Some(hole) if hole.from == begin => {
                    hole.from += pages;
                    Some(begin)
                }
                Some(hole) => {
                    let hole_end = hole.to;
                    hole.to = begin;
                    if let Some(slot) = self.holes.iter_mut().find(|h| h.size() == 0) {
                        slot.from = begin + pages;
                        slot.to = hole_end;
                    } else {
                        return None;
                    };
                    Some(begin)
                }
                _ => None,
            }
        } else {
            match self.holes.iter_mut().filter(|h| h.size() >= pages).max() {
                Some(hole) => {
                    let start = hole.from;
                    hole.from += pages;
                    Some(start)
                }
                _ => None,
            }
        }
    }

    fn dealloc(&mut self, begin: u16, size: u16) {
        if let Some(hole) = self.holes.iter_mut().find(|h| h.to == begin) {
            hole.to += size;
        } else if let Some(hole) = self.holes.iter_mut().find(|h| h.from == begin + size) {
            hole.from = begin;
        } else if let Some(slot) = self.holes.iter_mut().find(|h| h.size() == 0) {
            slot.from = begin;
            slot.to = begin + size;
        }
    }

    fn find_ref(&mut self, key: &[u8]) -> Result<Option<RecordRef>, Error<E>> {
        assert!(!key.is_empty());
        if !self.is_open {
            return Err(Error::StoreClosed);
        }
        let mut buf = [0; MAX_PAGE_SIZE];
        let hash = RecordRef::hash(key);
        for skip in 0..REFS_LEN {
            if let Some(rec_ref) = self
                .refs
                .iter()
                .filter(|r| r.hash == hash)
                .nth(skip as usize)
                .copied()
            {
                self.adapter
                    .read(
                        rec_ref.page * A::PAGE_SIZE,
                        &mut buf[0..A::PAGE_SIZE as usize],
                    )
                    .map_err(Error::AdapterError)?;
                let key_len = buf[0] as usize;
                if &buf[1..(key_len + 1)] == key {
                    return Ok(Some(rec_ref));
                }
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }

    fn save_ref(&mut self, rec_ref: RecordRef) -> Result<(), Error<E>> {
        let mut buf = [0; REF_SIZE as usize];
        rec_ref.serialize(&mut buf);
        let offset = A::PAGE_SIZE + rec_ref.idx * REF_SIZE;
        self.adapter
            .write(offset as u16, &buf)
            .map_err(Error::AdapterError)
    }

    fn reset(&mut self) {
        for (idx, rec_ref) in self.refs.iter_mut().enumerate() {
            rec_ref.idx = idx as u16;
            rec_ref.page = 0;
        }
        for (idx, hole) in self.holes.iter_mut().enumerate() {
            hole.idx = idx;
            hole.from = 0;
        }
        self.holes[0].from = self.index_pages + 1;
        self.holes[0].to = A::TOTAL_PAGES;
    }
}



pub fn pages(self, page_size: u16) -> u16 {
    let pages = self.len / page_size;
    if self.len - (pages * page_size) > 0 {
        pages + 1
    } else {
        pages
    }
}

pub fn read(buf: &[u8]) -> RecordRef {
    assert!(buf.len() >= 4);
    let val = BigEndian::read_u32(&buf[0..4]);
    let hash = val as u16 & 2047;
    let len = (val >> 11) as u16 & 4095;
    let page = (val >> 23) as u16 & 511;
    RecordRef {
        idx: 0,
        len,
        page,
        hash,
    }
}

pub fn write(self, buf: &mut [u8]) {
    assert!(buf.len() >= 4);
    let val = (self.hash as u32 & 2047)
        | (self.len as u32 & 4095) << 11
        | (self.page as u32 & 511) << 23;
    BigEndian::write_u32(&mut buf[0..4], val);
}