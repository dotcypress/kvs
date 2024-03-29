use crate::Address;

pub mod paged;
pub mod ram;
pub mod spi;

pub trait StoreAdapter {
    type Error;

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error>;
    fn max_address(&self) -> Address;
}
