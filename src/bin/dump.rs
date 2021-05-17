extern crate kvs;

use kvs::*;
use std::io::*;

const SIZE: usize = 1024;
const BUCKETS: usize = 32;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<SIZE>, BUCKETS, SLOTS>;

fn main() {
    let started = std::time::SystemTime::now();
    let mut store = MemoryStore::create(MemoryAdapter::new([0; SIZE]), 0xf00d).unwrap();
    for _ in 0..BUCKETS - 10 {
        store
            .insert(
                format!("{:?}", started.elapsed().unwrap()).as_bytes(),
                b"lorem ipsum",
            )
            .unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
