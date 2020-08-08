use kvs::{Error, KVStore, StoreAdapter};

struct MockAdapter {
  pub mem: Vec<u8>,
}

impl MockAdapter {
  pub fn new(mem_size: usize) -> Self {
    Self {
      mem: vec![0xff; mem_size],
    }
  }
}

impl StoreAdapter for MockAdapter {
  const MAGIC: [u8; 3] = *b"kvs";
  const PAGES: u16 = 1;
  const PAGE_SIZE: u32 = 4_096;
  type Error = ();

  fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
    let offset = addr as usize;
    buf.copy_from_slice(&self.mem[offset..(offset + buf.len())]);
    Ok(())
  }

  fn write(&mut self, addr: u32, data: &[u8]) -> Result<(), Self::Error> {
    let offset = addr as usize;
    &mut self.mem[offset..(offset + data.len())].copy_from_slice(data);
    Ok(())
  }
}

#[test]
fn test_open() {
  let store = KVStore::open(MockAdapter::new(2048), true).unwrap();
  let adapter = store.close();
  KVStore::open(adapter, false).unwrap();
}

#[test]
fn test_insert() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert(b"foo", b"bar").unwrap();
  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 3);
  assert_eq!(&buf[..n as usize], b"bar");
}

#[test]
fn test_reopen() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert(b"foo", b"bar").unwrap();

  let adapter = store.close();
  let mut store = KVStore::open(adapter, false).unwrap();

  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 3);
  assert_eq!(&buf[..n as usize], b"bar");
}

#[test]
fn test_reinsert() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert(b"foo", b"one").unwrap();
  store.insert(b"foo", b"two").unwrap();
  store.insert(b"foo", b"three").unwrap();
  store.insert(b"foo", b"bar").unwrap();
  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 3);
  assert_eq!(&buf[..n as usize], b"bar");
}

#[test]
fn test_insert_alloc() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store
    .insert_with_capacity(b"foo", b"bar", Some(16))
    .unwrap();
  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 16);
  assert_eq!(&buf[..n as usize], b"bar");
}

#[test]
fn test_patch() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store
    .insert_with_capacity(b"foo", b"aar", Some(16))
    .unwrap();
  let (len, cap) = store.patch(b"foo", 0, b"b").unwrap();

  assert_eq!(len, 3);
  assert_eq!(cap, 16);

  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 16);
  assert_eq!(&buf[..n as usize], b"bar");
}

#[test]
fn test_append() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 7);
  assert_eq!(&buf[..n as usize], b"bar");

  let (len, cap) = store.patch(b"foo", 3, b" baz").unwrap();
  assert_eq!(len, 7);
  assert_eq!(cap, 7);

  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 7);
  assert_eq!(len, 7);
  assert_eq!(cap, 7);
  assert_eq!(&buf[..n as usize], b"bar baz");
}

#[test]
fn test_patch_with_hole() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
  store.load(b"foo", &mut buf).unwrap();
  let err = store.patch(b"foo", 4, b"baz").unwrap_err();

  assert_eq!(err, Error::InvalidPatchOffset);
}

#[test]
fn test_patch_realloc() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
  let (len, cap) = store.patch(b"foo", 3, b" baz bar").unwrap();
  assert_eq!(len, 11);
  assert_eq!(cap, 11);

  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 11);
  assert_eq!(len, 11);
  assert_eq!(cap, 11);
  assert_eq!(&buf[..n as usize], b"bar baz bar");
}

#[test]
fn test_append_realloc() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
  let (len, cap) = store.append(b"foo", b" baz bar").unwrap();
  assert_eq!(len, 11);
  assert_eq!(cap, 11);

  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 11);
  assert_eq!(len, 11);
  assert_eq!(cap, 11);
  assert_eq!(&buf[..n as usize], b"bar baz bar");
}

#[test]
fn test_remove() {
  let mut buf = [0; 255];
  let mut store = KVStore::open(MockAdapter::new(2048), true).unwrap();

  store.insert(b"foo", b"bar").unwrap();
  store.remove(b"foo").unwrap();
  let err = store.load(b"foo", &mut buf).unwrap_err();
  assert_eq!(err, Error::KeyNofFound);

  store
    .insert_with_capacity(b"foo", b"bar", Some(16))
    .unwrap();
  let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

  assert_eq!(n, 3);
  assert_eq!(len, 3);
  assert_eq!(cap, 16);
  assert_eq!(&buf[..n as usize], b"bar");
}
