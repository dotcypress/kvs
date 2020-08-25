#![no_std]

mod config;
mod hole;
mod record;
mod store;

pub use store::KVStore;

pub trait StoreAdapter {
    const MAGIC: [u8; 4];
    const PAGES: u16;
    const PAGE_SIZE: u32;

    type Error;
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error>;

    fn write_paged(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
        let page_offset = addr % Self::PAGE_SIZE;
        if page_offset + data.len() as u32 <= Self::PAGE_SIZE {
            return self.write(addr, data);
        }

        let mut offset = 0;
        let mut chunk = Self::PAGE_SIZE - page_offset;
        while chunk > 0 {
            self.write(addr + offset, &data[offset as usize..(offset + chunk) as usize])?;
            offset += chunk;
            chunk = u32::min(Self::PAGE_SIZE, data.len() as u32 - offset);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    AdapterError(E),
    InvalidVersion,
    InvalidCapacity,
    InvalidPatchOffset,
    StoreNotFound,
    StoreClosed,
    StoreOverflow,
    KeyNofFound,
    Overread,
}
