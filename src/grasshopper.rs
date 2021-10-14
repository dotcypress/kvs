use byteorder::{BigEndian, ByteOrder};
use hash32::{Hasher, Murmur3Hasher};

#[derive(Default)]
pub struct Grasshopper<const SIZE: usize> {
    hops: usize,
    hash: u16,
    key_len: u32,
    token: u32,
}

impl<const SIZE: usize> Grasshopper<SIZE> {
    pub fn new(hops: usize, nonce: u16, key: &[u8]) -> Self {
        let mut hasher = Murmur3Hasher::default();

        if nonce != 0 {
            let mut nonce_buf = [0; 2];
            BigEndian::write_u16(&mut nonce_buf, nonce);
            hasher.write(&nonce_buf);
        }

        hasher.write(key);
        let token = hasher.finish();

        Self {
            hops,
            token,
            hash: token as u16,
            key_len: key.len() as u32,
        }
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
        let mut buf = [0, 0, 0, 0, self.key_len as u8];
        BigEndian::write_u32(&mut buf, self.token);
        hasher.write(&buf);

        self.token = hasher.finish();
        self.hops -= 1;

        Some(self.token as usize % SIZE)
    }
}

impl<const SIZE: usize> ExactSizeIterator for Grasshopper<SIZE> {
    fn len(&self) -> usize {
        SIZE
    }
}
