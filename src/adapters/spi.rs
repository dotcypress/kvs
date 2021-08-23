use core::fmt::Debug;

use byteorder::{BigEndian, ByteOrder};
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use crate::adapters::*;
pub enum Command {
    WriteStatusRegister = 0x01,
    Write = 0x02,
    Read = 0x03,
    WriteDisable = 0x04,
    ReadStatusRegister = 0x05,
    WriteEnable = 0x06,
}

pub enum Error<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> {
    ChipSelectError(CS::Error),
    TransferError(<SPI as spi::Transfer<u8>>::Error),
    WriteError(<SPI as spi::Write<u8>>::Error),
}

impl<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin> Debug for Error<SPI, CS> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ChipSelectError(_) => write!(f, "ChipSelect Error"),
            Self::TransferError(_) => write!(f, "SPI Transfer Error"),
            Self::WriteError(_) => write!(f, "SPI Write Error"),
        }
    }
}

#[derive(Debug)]
pub struct SpiStoreAdapter<
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
    const ADDR_BYTES: usize,
> {
    spi: SPI,
    cs: CS,
    max_addr: Address,
    min_addr: Address,
}

impl<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin, const ADDR_BYTES: usize>
    SpiStoreAdapter<SPI, CS, ADDR_BYTES>
{
    pub fn new(spi: SPI, cs: CS, min_addr: Address, max_addr: Address) -> Self {
        Self {
            spi,
            cs,
            max_addr,
            min_addr,
        }
    }

    pub fn release(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    pub fn read_status_register(&mut self) -> Result<u8, Error<SPI, CS>> {
        self.transaction(|spi| {
            spi.transfer(&mut [Command::ReadStatusRegister as u8, 0])
                .map(|buf| buf[1])
                .map_err(Error::TransferError)
        })
    }

    pub fn write_status_register(&mut self, status: u8) -> Result<(), Error<SPI, CS>> {
        self.transaction(|spi| {
            spi.transfer(&mut [Command::WriteStatusRegister as u8, status])
                .map_err(Error::TransferError)?;
            Ok(())
        })
    }

    pub fn transaction<RES, TX: FnOnce(&mut SPI) -> Result<RES, Error<SPI, CS>>>(
        &mut self,
        tx: TX,
    ) -> Result<RES, Error<SPI, CS>> {
        self.cs.set_low().map_err(Error::ChipSelectError)?;
        let res = tx(&mut self.spi);
        self.cs.set_high().map_err(Error::ChipSelectError).and(res)
    }

    fn mem_cmd(cmd: Command, addr: Address) -> [u8; 5] {
        assert!(ADDR_BYTES > 0 && ADDR_BYTES <= 4);

        let mut buf = [0; 5];
        buf[0] = cmd as u8;
        match ADDR_BYTES {
            1 => buf[1] = addr as u8,
            2 => BigEndian::write_u16(&mut buf[1..], addr as u16),
            3 => BigEndian::write_u24(&mut buf[1..], addr as u32),
            4 => BigEndian::write_u32(&mut buf[1..], addr as u32),
            _ => unreachable!(),
        };
        buf
    }
}

impl<SPI: spi::Transfer<u8> + spi::Write<u8>, CS: OutputPin, const ADDR_BYTES: usize> StoreAdapter
    for SpiStoreAdapter<SPI, CS, ADDR_BYTES>
{
    type Error = Error<SPI, CS>;

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
        let addr = addr + self.min_addr;
        assert!(!buf.is_empty() && addr + buf.len() < self.max_addr);

        self.transaction(|spi| {
            let mut cmd_buf = Self::mem_cmd(Command::Read, addr);
            spi.transfer(&mut cmd_buf[..ADDR_BYTES + 1])
                .and_then(|_| spi.transfer(buf))
                .map_err(Error::TransferError)?;

            Ok(())
        })
    }

    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error> {
        let addr = addr + self.min_addr;
        assert!(!data.is_empty() && addr + data.len() < self.max_addr);

        self.transaction(|spi| {
            spi.write(&[Command::WriteEnable as u8])
                .map_err(Error::WriteError)
        })?;

        self.transaction(|spi| {
            let mut cmd_buf = Self::mem_cmd(Command::Write, addr);
            spi.write(&mut cmd_buf[..ADDR_BYTES + 1])
                .and_then(|_| spi.write(&data))
                .map_err(Error::WriteError)
        })
    }

    fn max_address(&self) -> Address {
        self.max_addr
    }
}
