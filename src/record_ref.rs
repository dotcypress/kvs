use byteorder::{BigEndian, ByteOrder};
use hash32::{Hasher, Murmur3Hasher};

#[derive(Debug, Default, Copy, Clone)]
pub struct RecordRef {
    pub idx: u16,
    pub hash: u16,
    pub page: u16,
    pub len: u16,
}

impl RecordRef {
    pub fn hash(buf: &[u8]) -> u16 {
        let mut hasher = Murmur3Hasher::default();
        hasher.write(buf);
        hasher.finish() as u16 & 2047
    }

    pub fn active(self) -> bool {
        self.page != 0
    }

    pub fn pages(self, page_size: u16) -> u16 {
        let pages = self.len / page_size;
        if self.len - (pages * page_size) > 0 {
            pages + 1
        } else {
            pages
        }
    }

    pub fn deserialize(buf: &[u8]) -> RecordRef {
        assert!(buf.len() >= 4);
        let val = BigEndian::read_u32(&buf[0..4]);
        let hash = val as u16 & 2047;
        let len = (val >> 11) as u16 & 4095;
        let page = (val >> 23) as u16 & 511;
        RecordRef {
            idx: 0,
            len,
            page,
            hash,
        }
    }

    pub fn serialize(self, buf: &mut [u8]) {
        assert!(buf.len() >= 4);
        let val = (self.hash as u32 & 2047)
            | (self.len as u32 & 4095) << 11
            | (self.page as u32 & 511) << 23;
        BigEndian::write_u32(&mut buf[0..4], val);
    }
}
