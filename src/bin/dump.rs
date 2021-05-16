extern crate kvs;

use kvs::*;
use std::io::Write;

type MemoryStore = KVStore<MemoryAdapter<256>, 8, 8>;

fn main() {
    let mut store = MemoryStore::create(MemoryAdapter::new([0; 256]), 0xf00d).unwrap();
    store.insert(b"adapter", b"MemoryAdapter").unwrap();
    store.insert(b"size", b"256").unwrap();
    store.insert(b"magic", b"f00d").unwrap();
    store.insert(b"engine", b"kvs").unwrap();
    store.insert(b"date", b"Sun May 16 2021").unwrap();
    store.insert(b"foo", b"lorem ipsum").unwrap();
    store.insert(b"bar", b"dolor sit amet").unwrap();
    store.insert(b"baz", b"consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna").unwrap();

    std::io::stdout().write_all(&store.close().memory).ok();
}
