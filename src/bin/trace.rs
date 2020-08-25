extern crate kvs;

use kvs::*;

#[derive(Debug)]
struct MemAdapter {
    pub mem: Vec<u8>,
}

impl MemAdapter {
    pub fn new() -> Self {
        Self {
            mem: vec![0xff; MemAdapter::PAGES as usize * MemAdapter::PAGE_SIZE as usize],
        }
    }
}

impl StoreAdapter for MemAdapter {
    const MAGIC: [u8; 4] = *b"kvs1";
    const PAGES: u16 = 256;
    const PAGE_SIZE: u32 = 16;
    type Error = ();

    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        let offset = addr as usize;
        println!(
            "\tmem  read: {}\t@{}..{}",
            buf.len(),
            offset,
            offset + buf.len()
        );
        buf.copy_from_slice(&self.mem[offset..(offset + buf.len())]);
        Ok(())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
        let offset = addr as usize;
        println!(
            "\tmem write: {}\t@{}..{}",
            data.len(),
            offset,
            offset + data.len()
        );
        let window = &mut self.mem[offset..(offset + data.len())];
        window.copy_from_slice(data);
        Ok(())
    }
}

fn main() {
    let mut buf = [0; 255];
    let mut store = KVStore::open(MemAdapter::new(), true).unwrap();
    store.insert(b"foo", b"lorem-ipsum").unwrap();

    let adapter = store.close();
    let mut store = KVStore::open(adapter, false).unwrap();

    let res = store.load(b"foo", &mut buf).unwrap();
    println!(
        "{:?} {}",
        res,
        String::from_utf8_lossy(&buf[..res.0 as usize])
    );

    store.patch(b"foo", 5, b" ").unwrap();

    let res = store.load(b"foo", &mut buf).unwrap();
    println!(
        "{:?} {}",
        res,
        String::from_utf8_lossy(&buf[..res.0 as usize])
    );
}
