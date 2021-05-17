#![no_std]

mod alloc;
mod grasshopper;
mod store;

use modular_bitfield::prelude::*;

pub use alloc::*;
pub use grasshopper::*;
pub use store::*;

pub const MAX_KEY_LEN: usize = 128;
pub const MAX_VALUE_LEN: usize = 32 * 1024;

const BUCKET_BATCH_SIZE: usize = 32;

#[derive(Debug)]
pub struct Bucket {
    index: usize,
    raw: RawBucket,
}

impl Bucket {
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn addr(&self) -> usize {
        self.raw.addr() as usize
    }

    pub fn key_len(&self) -> usize {
        self.raw.key_len() as usize
    }

    pub fn val_len(&self) -> usize {
        self.raw.val_len() as usize
    }

    pub fn record_len(&self) -> usize {
        self.key_len() + self.val_len()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    AdapterError(E),
    IndexOverflow,
    InvalidCapacity,
    InvalidPatchOffset,
    KeyNofFound,
    ReadOnlyStore,
    StoreNotFound,
    StoreOverflow,
}

#[bitfield]
pub(crate) struct RawStoreHeader {
    magic: B32,
    buckets: B16,
    #[skip]
    dummy: B16,
}

#[bitfield]
#[derive(Default, Debug, Clone)]
pub(crate) struct RawBucket {
    in_use: bool,
    val_len: B15,
    key_len: B8,
    addr: B24,
    hash: B16,
}

pub trait StoreAdapter {
    type Error;

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn space(&self) -> usize;
}

pub struct PagedMemoryAdapter<A, const OFFSET: usize, const PAGES: usize, const PAGE_SIZE: usize>
where
    A: StoreAdapter,
{
    inner: A,
}

impl<A, const OFFSET: usize, const PAGES: usize, const PAGE_SIZE: usize>
    PagedMemoryAdapter<A, OFFSET, PAGES, PAGE_SIZE>
where
    A: StoreAdapter,
{
    pub fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl<A, const OFFSET: usize, const PAGES: usize, const PAGE_SIZE: usize> StoreAdapter
    for PagedMemoryAdapter<A, OFFSET, PAGES, PAGE_SIZE>
where
    A: StoreAdapter,
{
    type Error = A::Error;

    fn space(&self) -> usize {
        self.inner.space()
    }

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read(addr + OFFSET, buf)
    }

    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error> {
        let addr = addr + OFFSET;
        let page_offset = addr % PAGE_SIZE;
        if page_offset + data.len() <= PAGE_SIZE {
            return self.inner.write(addr, data);
        }

        let mut offset = 0;
        let mut chunk = PAGE_SIZE - page_offset;
        while chunk > 0 {
            self.inner
                .write(addr + offset, &data[offset..(offset + chunk)])?;
            offset += chunk;
            chunk = usize::min(PAGE_SIZE, data.len() - offset);
        }

        Ok(())
    }
}

pub struct MemoryAdapter<const SIZE: usize> {
    pub memory: [u8; SIZE],
}

impl<const SIZE: usize> Default for MemoryAdapter<SIZE> {
    fn default() -> Self {
        Self::new([0; SIZE])
    }
}

impl<const SIZE: usize> MemoryAdapter<SIZE> {
    pub fn new(memory: [u8; SIZE]) -> Self {
        Self { memory }
    }

    pub fn free(self) -> [u8; SIZE] {
        self.memory
    }
}

impl<const SIZE: usize> StoreAdapter for MemoryAdapter<SIZE> {
    type Error = ();

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        if addr + buf.len() > SIZE {
            return Err(());
        }
        buf.copy_from_slice(&self.memory[addr..(addr + buf.len())]);
        Ok(())
    }

    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error> {
        if addr + data.len() > SIZE {
            return Err(());
        }
        self.memory[addr..(addr + data.len())].copy_from_slice(data);
        Ok(())
    }

    fn space(&self) -> usize {
        SIZE
    }
}
