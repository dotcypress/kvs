use byteorder::{BigEndian, ByteOrder};
use hash32::{Hasher, Murmur3Hasher};

#[derive(Default)]
pub struct Grasshopper<const SIZE: usize> {
    hops: usize,
    hash: u16,
    token: u16,
}

impl<const SIZE: usize> Grasshopper<SIZE> {
    pub fn new(hops: usize, key: &[u8]) -> Self {
        let mut hasher = Murmur3Hasher::default();
        hasher.write(key);
        let hash = hasher.finish() as u16;
        let token = hash;

        Self { hash, token, hops }
    }

    pub fn hash(&self) -> u16 {
        self.hash
    }
}

impl<const SIZE: usize> Iterator for Grasshopper<SIZE> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.hops == 0 {
            return None;
        }

        let mut hasher = Murmur3Hasher::default();
        let mut buf = [0, 0];
        BigEndian::write_u16(&mut buf, self.token);
        hasher.write(&buf);
        self.token = hasher.finish() as u16;

        self.hops -= 1;
        Some(self.token as usize % SIZE)
    }
}
