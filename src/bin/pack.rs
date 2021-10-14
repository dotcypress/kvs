extern crate kvs;

use kvs::*;
use std::collections::HashMap;

const BUCKETS: usize = 32;
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

fn main() {
    let keys = KEYS.iter().map(|x| x.to_string()).collect();
    let res: Vec<(usize, u16)> = (0..u16::MAX)
        .map(|nonce| (pack(&keys, nonce), nonce))
        .collect();
    let min = res.iter().min();
    let max = res.iter().max();
    println!("{:?} - {:?}", min.unwrap(), max.unwrap());
}

fn pack(keys: &Vec<String>, nonce: u16) -> usize {
    let mut buckets = HashMap::new();
    for key in keys {
        let hopper = Grasshopper::<{ BUCKETS }>::new(1_000, nonce, key.as_bytes());
        for (hop, bucket) in hopper.enumerate() {
            if buckets.get(&bucket).is_none() {
                buckets.insert(bucket, hop + 1);
                break;
            }
        }
    }
    *buckets.values().max().unwrap()
}
