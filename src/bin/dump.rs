extern crate kvs;

use kvs::*;
use std::io::*;

const SIZE: usize = 1024;
const BUCKETS: usize = 32;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<SIZE>, BUCKETS, SLOTS>;

const KEY_COLLISIONS: [&str; 16] = [
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
];

fn main() {
    let mut store = MemoryStore::create(MemoryAdapter::new([0; SIZE]), StoreOptions::new(0xf00d, 32)).unwrap();
    for key in KEY_COLLISIONS.iter() {
        store.insert(key.as_bytes(), key.as_bytes()).unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
