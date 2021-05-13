use hash32::{Hasher, Murmur3Hasher};

pub fn _hash(key: &[u8]) -> u32 {
    let mut hasher = Murmur3Hasher::default();
    hasher.write(key);
    hasher.finish()
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RecordRef {
    key_hash: u32,
    record_addr: usize,
}
#[derive(Debug, Default, Copy, Clone)]
pub struct Record {
    pub ref_idx: usize,
    pub addr: usize,
    pub key_len: usize,
    pub value_len: usize,
    pub value_cap: usize,
}