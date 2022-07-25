extern crate kvs;

use kvs::adapters::StoreAdapter;
use kvs::*;

#[derive(Debug)]
struct TraceMemoryAdapter {
    pub memory: Vec<u8>,
}

impl StoreAdapter for TraceMemoryAdapter {
    type Error = ();

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
        if buf.len() + addr > self.memory.len() {
            return Err(());
        }
        buf.copy_from_slice(&self.memory[addr..(addr + buf.len())]);
        println!(
            "R: {:3} [0x{:03x}..0x{:03x}] {:02x?}",
            buf.len(),
            addr,
            addr + buf.len(),
            if buf.len() > 16 { &buf[..16] } else { buf }
        );
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
            if data.len() > 16 { &data[..16] } else { data }
        );
        self.memory[addr..(addr + data.len())].copy_from_slice(data);
        Ok(())
    }

    fn max_address(&self) -> Address {
        self.memory.len()
    }
}

#[cfg(feature = "crud")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
struct LogEntry {
    ts: usize,
    val: u8,
}

fn main() {
    #[cfg(feature = "crud")]
    {
        const BUCKETS: usize = 4;
        type CrudStore = KVStore<TraceMemoryAdapter, BUCKETS, 4>;
        let mut store = CrudStore::open(
            TraceMemoryAdapter {
                memory: vec![0; 0x1000],
            },
            StoreConfig::new(0xf00d, 8),
            true,
        )
        .unwrap();
        store
            .create::<LogEntry, 32>(b"100500", &LogEntry { ts: 420, val: 70 })
            .unwrap();
        let entry = store.read::<LogEntry, 32>(b"100500").unwrap();
        println!("entry: {:?}", entry);
    }
}
