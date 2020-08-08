extern crate kvs;

use kvs::*;

#[derive(Debug)]
struct MemAdapter {
    pub mem: Vec<u8>,
}

impl MemAdapter {
    pub fn new(mem_size: usize) -> Self {
        Self {
            mem: vec![0xff; mem_size],
        }
    }
}

impl StoreAdapter for MemAdapter {
    const MAGIC: [u8; 3] = *b"kvs";
    const PAGES: u16 = 1;
    const PAGE_SIZE: u32 = 4_096;
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
    let mut store = KVStore::open(MemAdapter::new(10_000), true).unwrap();
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
