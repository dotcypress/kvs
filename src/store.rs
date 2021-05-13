use byteorder::{BigEndian, ByteOrder};

use crate::hole::Hole;
use crate::{Error, StoreAdapter};

const VERSION: u8 = 2;
const HEADER_SIZE: usize = 8;
const RECORD_REF_SIZE: usize = 8;
const BUFFER_SIZE: usize = 256;
const MAX_KEY_SIZE: usize = 128;

pub trait StoreIndex {
    fn reset(&mut self);
}

pub struct NoIndex;

impl StoreIndex for NoIndex {
    fn reset(&mut self) {}
}

pub struct KVStore<A, const CAPACITY: usize, const HOLES: usize>
where
    A: StoreAdapter,
{
    adapter: A,
    magic: u32,
    seed: u8,
    buf: [u8; BUFFER_SIZE],
    holes: [Hole; HOLES],
    ref_map: [bool; CAPACITY],
}

impl<E, A, const CAPACITY: usize, const HOLES: usize> KVStore<A, CAPACITY, HOLES>
where
    A: StoreAdapter<Error = E>,
{
    pub fn open(adapter: A, magic: u32, create_if_empty: bool) -> Result<Self, Error<E>> {
        assert!(CAPACITY % 8 == 0, "Capacity must be multiple of 8");

        let mut adapter = adapter;
        let adapter_space = adapter.space();
        let ref_map_size = CAPACITY / 8;
        let meta_size = HEADER_SIZE + ref_map_size + RECORD_REF_SIZE * CAPACITY;

        let mut holes = [Hole::default(); HOLES];
        holes[0] = Hole::new(meta_size, adapter_space);

        let mut buf = [0; HEADER_SIZE];
        adapter.read(0, &mut buf).map_err(Error::AdapterError)?;

        let store_magic = BigEndian::read_u32(&buf[0..4]);
        let store_version = buf[4];
        let store_capacity = BigEndian::read_u16(&buf[6..8]);
        let seed = buf[5];

        let mut store = KVStore {
            adapter,
            magic,
            holes,
            seed,
            buf: [0; BUFFER_SIZE],
            ref_map: [false; CAPACITY],
        };

        if store_magic != magic {
            return if create_if_empty {
                store.format()?;
                Ok(store)
            } else {
                Err(Error::StoreNotFound)
            };
        }

        if store_version != VERSION {
            return Err(Error::InvalidVersion);
        }

        if store_capacity as usize != CAPACITY {
            return Err(Error::InvalidCapacity);
        }

        //TODO: rotate seed

        store.load_index()?;

        Ok(store)
    }

    pub fn close(self) -> A {
        self.adapter
    }

    pub fn format(&mut self) -> Result<(), Error<E>> {
        self.seed += 1;
        self.holes.iter_mut().for_each(|h| h.reset());
        self.ref_map.iter_mut().for_each(|r| *r = false);

        let index_size = CAPACITY * RECORD_REF_SIZE;
        let meta_size = HEADER_SIZE + index_size;
        self.holes[0] = Hole::new(meta_size, self.adapter.space());

        BigEndian::write_u32(&mut self.buf[0..4], self.magic);
        self.buf[4] = VERSION;
        self.buf[5] = self.seed;
        BigEndian::write_u16(&mut self.buf[6..8], CAPACITY as u16);
        self.adapter
            .write(0, &self.buf[..HEADER_SIZE])
            .map_err(Error::AdapterError)
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error<E>> {
        self.insert_with_capacity(key, val, val.len())
    }

    pub fn insert_with_capacity(
        &mut self,
        key: &[u8],
        val: &[u8],
        capacity: usize,
    ) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_SIZE);
        assert!(!val.is_empty() || capacity >= val.len());
        Ok(())
    }

    fn load_index(&mut self) -> Result<(), Error<E>> {
        Ok(())
    }

    fn _alloc(&mut self, size: usize, addr: Option<usize>) -> Option<usize> {
        if let Some(addr) = addr {
            match self
                .holes
                .iter_mut()
                .find(|h| addr >= h.start && addr < h.end && h.size() - (addr - h.start) >= size)
            {
                Some(hole) if hole.start == addr => {
                    hole.start += size;
                    Some(addr)
                }
                Some(hole) => {
                    let hole_end = hole.end;
                    hole.end = addr;
                    if let Some(unused_hole) = self.holes.iter_mut().find(|h| h.size() == 0) {
                        unused_hole.start = addr + size;
                        unused_hole.end = hole_end;
                    } else {
                        return None;
                    };
                    Some(addr)
                }
                _ => None,
            }
        } else {
            let hole = self.holes.iter_mut().filter(|h| h.size() >= size).max()?;
            let start = hole.start;
            hole.start += size;
            Some(start)
        }
    }

    fn _free(&mut self, addr: usize, size: usize) {
        let hole_end = addr + size;
        if let Some(hole) = self.holes.iter_mut().find(|h| h.end == addr) {
            hole.end += size;
        } else if let Some(hole) = self.holes.iter_mut().find(|h| h.start == hole_end) {
            hole.start = addr;
        } else if let Some(slot) = self.holes.iter_mut().find(|h| h.size() == 0) {
            slot.start = addr;
            slot.end = hole_end;
        }
    }
}
