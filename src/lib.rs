#![no_std]

mod hole;
mod record_ref;
mod store;

pub use store::KVStore;

pub trait StoreAdapter {
    const MAGIC: [u8; 4];
    const TOTAL_PAGES: u16;
    const PAGE_SIZE: u16;

    const VERSION: u8 = 1;
    const MAX_KEY_LEN: usize = 16;
    const MAX_VAL_LEN: usize = 4095;

    type Error;

    fn write(&mut self, addr: u16, data: &[u8]) -> Result<(), Self::Error>;
    fn read(&mut self, addr: u16, buff: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum StoreError<E> {
    AdapterError(E),
    StoreNotFound,
    StoreClosed,
    Overflow,
    AppendFailed,
    IndexOverflow,
    ValueOverflow,
    KeyNofFound,
    InvalidVersion,
}
