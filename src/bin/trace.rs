extern crate kvs;

use kvs::*;

#[derive(Debug)]
struct TraceMemoryAdapter {
    pub memory: Vec<u8>,
}

impl TraceMemoryAdapter {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 0x1000],
        }
    }
}

impl StoreAdapter for TraceMemoryAdapter {
    type Error = ();

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
        if buf.len() + addr > self.memory.len() {
            return Err(());
        }
        println!(
            "R: {:3} [0x{:03x}..0x{:03x}] {:02x?}",
            buf.len(),
            addr,
            addr + buf.len(),
            if buf.len() > 16 { &buf[..16] } else { &buf }
        );
        buf.copy_from_slice(&self.memory[addr..(addr + buf.len())]);
        Ok(())
    }

    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error> {
        if addr + data.len() > self.memory.len() {
            return Err(());
        }
        println!(
            "W: {:3} [0x{:03x}..0x{:03x}] {:02x?}",
            data.len(),
            addr,
            addr + data.len(),
            if data.len() > 16 { &data[..16] } else { &data }
        );
        self.memory[addr..(addr + data.len())].copy_from_slice(data);
        Ok(())
    }

    fn max_address(&self) -> Address {
        self.memory.len()
    }
}

fn main() {
    const BUCKETS: usize = 256;
    type TraceStore = KVStore<TraceMemoryAdapter, BUCKETS, 24>;
    let mut store =
        TraceStore::create(TraceMemoryAdapter::new(), StoreOptions::new(0xf00d, 24)).unwrap();
    store
        .insert(b"foo", b"consectetur adipiscing elit")
        .unwrap();
    store.insert(b"bar", b"dolor sit amet").unwrap();
    store.insert(b"foo", b"lorem ipsum").unwrap();
}
