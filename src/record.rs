use hash32::{Hasher, Murmur3Hasher};

pub fn hash_key(key: &[u8]) -> u16 {
    let mut hasher = Murmur3Hasher::default();
    hasher.write(key);
    hasher.finish() as u16 | 0b11
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Record {
    pub ref_idx: usize,
    pub addr: u32,
    pub key_len: u8,
    pub value_len: u16,
    pub value_cap: u16,
}

impl Record {
    pub fn size(&self) -> u32 {
        self.key_len as u32 + self.value_cap as u32 + 5
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RecordRef {
    pub data: u16,
}

impl Default for RecordRef {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordRef {
    pub fn new() -> Self {
        RecordRef { data: u16::MAX }
    }

    pub fn reset(&mut self) {
        self.data = u16::MAX;
    }

    pub fn load_hash(&mut self, hash: u16) {
        self.data = hash | 0b11;
    }

    pub fn get_hash(&self) -> u16 {
        self.data | 0b11
    }

    pub fn in_use(&self) -> bool {
        self.data & 0b1 == 0
    }

    pub fn is_deleted(&self) -> bool {
        self.data & 0b10 == 0
    }

    pub fn set_deleted(&mut self) {
        self.data = 0b1;
    }

    pub fn set_in_use(&mut self) {
        self.data = self.data >> 2 << 2 | 0b10;
    }
}
