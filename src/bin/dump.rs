extern crate kvs;

use fakedata_generator::*;
use kvs::*;
use std::io::*;

const SIZE: usize = 2000;
const BUCKETS: usize = 50;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<SIZE>, BUCKETS, SLOTS>;

fn main() {
    let mut store = MemoryStore::create(MemoryAdapter::new([0; SIZE]), 0xf00d).unwrap();
    for _ in 0..BUCKETS - 4 {
        store
            .insert(gen_ipv4().as_bytes(), gen_domain().as_bytes())
            .unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
