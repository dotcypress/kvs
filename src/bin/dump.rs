extern crate kvs;

use kvs::adapters::ram::MemoryAdapter;
use kvs::*;
use std::io::*;

const KEYS: [&str; 18] = [
    "key_29589",
    "key_34447",
    "key_42952",
    "key_45273",
    "key_69027",
    "key_93956",
    "key_126220",
    "key_153367",
    "key_155294",
    "key_176347",
    "key_187034",
    "key_217113",
    "key_218347",
    "key_221505",
    "key_227523",
    "key_231448",
    "foo",
    "bar",
];

const STORE_SIZE: usize = 512;
const BUCKETS: usize = 24;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

fn main() {
    let opts = StoreOptions::new(0x4b1d, 32);
    let mut store = MemoryStore::create(MemoryAdapter::default(), opts).unwrap();
    for (idx, key) in KEYS.iter().enumerate() {
        store.insert(key.as_bytes(), &[idx as u8]).unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
