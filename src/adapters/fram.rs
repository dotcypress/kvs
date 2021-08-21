use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use crate::*;

enum Command {
    ReadStatusRegister = 0b0000_0101,
    WriteStatusRegister = 0b0000_0001,
    Read = 0b0000_0011,
    WriteEnable = 0b0000_0110,
    Write = 0b0000_0010,
    WriteDisable = 0b0000_0100,
}

pub enum Error<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> {
    ChipSelectError(CS::Error),
    TransferError(<SPI as spi::Transfer<u8>>::Error),
    WriteError(<SPI as spi::Write<u8>>::Error),
}

#[derive(Debug)]
pub struct FramStoreAdapter<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> {
    spi: SPI,
    cs: CS,
    max_addr: Address,
}

impl<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> FramStoreAdapter<SPI, CS> {
    pub fn new(spi: SPI, cs: CS, max_addr: Address) -> Self {
        Self { spi, cs, max_addr }
    }

    pub fn spi_mut(&mut self) -> &mut SPI {
        &mut self.spi
    }

    pub fn cs_mut(&mut self) -> &mut CS {
        &mut self.cs
    }

    pub fn release(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    pub fn read_status(&mut self) -> Result<u8, Error<SPI, CS>> {
        self.transaction(|spi| {
            spi.transfer(&mut [Command::ReadStatusRegister as u8, 0])
                .map(|buf| buf[1])
                .map_err(Error::TransferError)
        })
    }

    pub fn write_status(&mut self, status: u8) -> Result<(), Error<SPI, CS>> {
        self.transaction(|spi| {
            spi.transfer(&mut [Command::WriteStatusRegister as u8, status])
                .map(|_| ())
                .map_err(Error::TransferError)
        })
    }

    fn transaction<RES, TX: FnOnce(&mut SPI) -> Result<RES, Error<SPI, CS>>>(
        &mut self,
        tx: TX,
    ) -> Result<RES, Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::ChipSelectError)?;
        let res = tx(&mut self.spi);
        self.cs.set_high().map_err(Error::ChipSelectError).and(res)
    }
}

impl<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> StoreAdapter
    for FramStoreAdapter<SPI, CS>
{
    type Error = Error<SPI, CS>;

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
        assert!(buf.len() > 0 && addr + buf.len() < self.max_addr);

        self.transaction(|spi| {
            let mut read = [
                Command::Read as u8,
                (addr >> 16) as u8,
                (addr >> 8) as u8,
                addr as u8,
            ];

            spi.transfer(&mut read)
                .and_then(|_| spi.transfer(buf))
                .map(|_| ())
                .map_err(Error::TransferError)
        })
    }

    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error> {
        assert!(data.len() > 0 && addr + data.len() < self.max_addr);

        self.transaction(|spi| {
            let mut write = [
                Command::Write as u8,
                (addr >> 16) as u8,
                (addr >> 8) as u8,
                addr as u8,
            ];
            let mut write_enable = [Command::WriteEnable as u8];
            let mut write_disable = [Command::WriteDisable as u8];

            spi.transfer(&mut write_enable)
                .and_then(|_| spi.transfer(&mut write))
                .map_err(Error::TransferError)
                .and_then(|_| spi.write(data).map_err(Error::WriteError))
                .and_then(|_| {
                    spi.transfer(&mut write_disable)
                        .map_err(Error::TransferError)
                })
                .map(|_| ())
        })
    }

    fn max_address(&self) -> Address {
        self.max_addr
    }
}
