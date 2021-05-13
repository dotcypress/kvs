#![no_std]

mod hole;
mod record;
mod store;

pub use store::KVStore;
pub use store::NoIndex;

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    AdapterError(E),
    StoreNotFound,
    InvalidVersion,
    InvalidCapacity,
    StoreOverflow,
    KeyNofFound,
    InvalidPatchOffset,
    Overread,
}

pub trait StoreAdapter {
    type Error;

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn space(&self) -> usize;
}

pub struct PagedStoreAdapter<A, const PAGES: usize, const PAGE_SIZE: usize>
where
    A: StoreAdapter,
{
    inner: A,
}

impl<A, const PAGES: usize, const PAGE_SIZE: usize> PagedStoreAdapter<A, PAGES, PAGE_SIZE>
where
    A: StoreAdapter,
{
    pub fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl<A, const PAGES: usize, const PAGE_SIZE: usize> StoreAdapter
    for PagedStoreAdapter<A, PAGES, PAGE_SIZE>
where
    A: StoreAdapter,
{
    type Error = A::Error;

    fn space(&self) -> usize {
        self.inner.space()
    }

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read(addr, buf)
    }

    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error> {
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

pub struct MemoryStoreAdapter<const SIZE: usize> {
    pub memory: [u8; SIZE],
}

impl<const SIZE: usize> Default for MemoryStoreAdapter<SIZE> {
    fn default() -> Self {
        Self::new([0; SIZE])
    }
}

impl<const SIZE: usize> MemoryStoreAdapter<SIZE> {
    pub fn new(memory: [u8; SIZE]) -> Self {
        Self { memory }
    }

    pub fn free(self) -> [u8; SIZE] {
        self.memory
    }
}

impl<const SIZE: usize> StoreAdapter for MemoryStoreAdapter<SIZE> {
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
