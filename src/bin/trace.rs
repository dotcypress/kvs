extern crate kvs;

use kvs::*;

const SIZE: usize = 1024;
pub const MAGIC: u32 = 0xf00d;
pub const CAPACITY: usize = 64;
pub const MAX_HOLES: usize = 64;

type TraceStore = KVStore<TraceMemoryAdapter, CAPACITY, MAX_HOLES>;

#[derive(Debug)]
struct TraceMemoryAdapter {
    pub memory: Vec<u8>,
}

impl TraceMemoryAdapter {
    pub fn new() -> Self {
        Self {
            memory: vec![0x00; SIZE],
        }
    }
}

impl StoreAdapter for TraceMemoryAdapter {
    type Error = ();

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        if buf.len() + addr > SIZE {
            return Err(());
        }
        println!(
            "\tmem  read: {}\t@{}..{}\t{:?}",
            buf.len(),
            addr,
            addr + buf.len(),
            buf
        );
        buf.copy_from_slice(&self.memory[addr..(addr + buf.len())]);
        Ok(())
    }

    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error> {
        if addr + data.len() > SIZE {
            return Err(());
        }
        println!(
            "\tmem write: {}\t@{}..{}\t{:?}",
            data.len(),
            addr,
            addr + data.len(),
            data
        );
        self.memory[addr..(addr + data.len())].copy_from_slice(data);
        Ok(())
    }

    fn space(&self) -> usize {
        SIZE
    }
}

fn main() {
    let mut store = TraceStore::open(TraceMemoryAdapter::new(), MAGIC, true).unwrap();
    store.insert(b"foo", b"lorem-ipsum").unwrap();

    let adapter = store.close();
    TraceStore::open(adapter, MAGIC, false).unwrap();
}
