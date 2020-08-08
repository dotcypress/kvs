//#![no_std]

mod hole;
mod record;
mod store;

pub use store::KVStore;

pub trait StoreAdapter {
    const MAGIC: [u8; 3];
    const PAGES: u16;
    const PAGE_SIZE: u32;

    type Error;

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
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