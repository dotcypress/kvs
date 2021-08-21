use crate::adapters::*;

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

  fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
      if addr + buf.len() > SIZE {
          return Err(());
      }
      buf.copy_from_slice(&self.memory[addr..(addr + buf.len())]);
      Ok(())
  }

  fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error> {
      if addr + data.len() > SIZE {
          return Err(());
      }
      self.memory[addr..(addr + data.len())].copy_from_slice(data);
      Ok(())
  }

  fn max_address(&self) -> Address {
      SIZE
  }
}
