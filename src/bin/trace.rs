extern crate kvs;

use kvs::*;

#[derive(Debug)]
struct TraceMemoryAdapter {
    pub memory: Vec<u8>,
}

impl TraceMemoryAdapter {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 0x10000],
        }
    }
}

impl StoreAdapter for TraceMemoryAdapter {
    type Error = ();

    fn read(&mut self, addr: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
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

    fn write(&mut self, addr: usize, data: &[u8]) -> Result<(), Self::Error> {
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

    fn space(&self) -> usize {
        self.memory.len()
    }
}

fn main() {
    const BUCKETS: usize = 0x1000;
    type TraceStore = KVStore<TraceMemoryAdapter, BUCKETS, 24>;
    let mut store = TraceStore::create(TraceMemoryAdapter::new(), 0xf00d).unwrap();
    store
        .insert(b"foo", b"consectetur adipiscing elit")
        .unwrap();
    store.insert(b"bar", b"dolor sit amet").unwrap();
    store.insert(b"foo", b"lorem ipsum").unwrap();

    use core::mem::size_of;
    type Dummy = MemoryAdapter<0>;
    println!("1: {} B", size_of::<KVStore<Dummy, BUCKETS, 1>>());
    println!("8: {} B", size_of::<KVStore<Dummy, BUCKETS, 8>>());
    println!("16: {} B", size_of::<KVStore<Dummy, BUCKETS, 16>>());
    println!("32: {} B", size_of::<KVStore<Dummy, BUCKETS, 32>>());
    println!("64: {} B", size_of::<KVStore<Dummy, BUCKETS, 64>>());
    println!("128: {} B", size_of::<KVStore<Dummy, BUCKETS, 128>>());
    println!("256: {} B", size_of::<KVStore<Dummy, BUCKETS, 256>>());
    println!("512: {} B", size_of::<KVStore<Dummy, BUCKETS, 512>>());
    println!("1024: {} B", size_of::<KVStore<Dummy, BUCKETS, 1024>>());
}
