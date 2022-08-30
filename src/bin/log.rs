extern crate kvs;

use kvs::adapters::ram::MemoryAdapter;
use kvs::*;
use std::io::*;

const STORE_SIZE: usize = 256;
const BUCKETS: usize = 4;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

fn main() {
    let cfg = StoreConfig::new(42, 32);
    let adapter = MemoryAdapter::default();
    let mut store = MemoryStore::open(adapter, cfg, true).unwrap();
    store.insert(b"log/cursor", &[0]).unwrap();
    store.alloc(b"log/data", 32, None).unwrap();
    for cursor in 0..32 {
        store.patch(b"log/cursor", 0, &[cursor as _]).unwrap();
        store
            .patch(b"log/data", cursor, &[cursor as u8 + 16])
            .unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
