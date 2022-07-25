extern crate kvs;

use kvs::adapters::ram::MemoryAdapter;
use kvs::*;
use std::io::*;

const KEYS: [&str; 16] = [
    "/bin/charlotte/big/great/year.jpg",
    "/bin/group/place/public.txt",
    "/bin/oliver/fact/year/place.txt",
    "/etc/able/woman/mia.mp4",
    "/etc/big/week.jpg",
    "/home/able/year.rar",
    "/home/early/group.jpg",
    "/isabella/time.mp4",
    "/sbin/young/eye/problem.xls",
    "/thing/charlotte.flv",
    "/tmp/amelia/emma/good.tar",
    "/usr/elijah/right.tar",
    "/usr/emma.rar",
    "/usr/olivia/different.jpg",
    "/var/aria/high.zip",
    "/var/life/time/oliver.tar",
];

const STORE_SIZE: usize = 1024;
const BUCKETS: usize = 32;
const SLOTS: usize = 1;

type MemoryStore = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

fn main() {
    let cfg = StoreConfig::new(0x796e6974, 32).nonce(34);
    let mut store = MemoryStore::open(MemoryAdapter::default(), cfg, true).unwrap();
    for key in KEYS.iter() {
        store.insert(key.as_bytes(), key.as_bytes()).unwrap();
    }

    stdout().write_all(&store.close().memory).ok();
}
