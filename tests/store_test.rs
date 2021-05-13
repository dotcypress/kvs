use kvs::{KVStore, MemoryStoreAdapter};

mod tiny {
    use crate::*;

    pub const MAGIC: u32 = 0x4b1d;
    pub const STORE_SIZE: usize = 16 * 1024;
    pub const CAPACITY: usize = 64;
    pub const MAX_HOLES: usize = 64;

    pub type Adapter = MemoryStoreAdapter<STORE_SIZE>;
    pub type Store = KVStore<Adapter, CAPACITY, MAX_HOLES>;

    pub fn create_store() -> Store {
        Store::open(MemoryStoreAdapter::default(), MAGIC, true).unwrap()
    }
}

#[test]
fn test_create_invalid_store() {
    type TooSmallStore = KVStore<MemoryStoreAdapter<2>, { tiny::CAPACITY }, { tiny::MAX_HOLES }>;

    let store = TooSmallStore::open(MemoryStoreAdapter::default(), tiny::MAGIC, true);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::AdapterError(()));
    }
}

#[test]
fn test_create_store() {
    tiny::create_store();
}

#[test]
fn test_reopen_store() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(adapter, tiny::MAGIC, false);
    assert!(store.is_ok());
}

#[test]
fn test_reopen_store_with_invalid_magic() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(adapter, tiny::MAGIC + 1, false);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::StoreNotFound);
    }
}

#[test]
fn test_reopen_store_with_invalid_capacity() {
    let adapter = tiny::create_store().close();

    type WrongCapacityStore = KVStore<
        MemoryStoreAdapter<{ tiny::STORE_SIZE }>,
        { tiny::CAPACITY * 2 },
        { tiny::MAX_HOLES },
    >;

    let store = WrongCapacityStore::open(adapter, tiny::MAGIC, false);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::InvalidCapacity);
    }
}

#[test]
fn test_reopen_store_with_invalid_version() {
    let mut adapter = tiny::create_store().close();
    adapter.memory[4] = 0xff;

    let store = tiny::Store::open(adapter, tiny::MAGIC, false);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::InvalidVersion);
    }
}

#[test]
fn test_insert() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();
    // let mut buf = [0; 255];
    // let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

    //   assert_eq!(n, 3);
    //   assert_eq!(len, 3);
    //   assert_eq!(cap, 3);
    //   assert_eq!(&buf[..n as usize], b"bar");
}

// #[test]
// fn test_reopen() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert(b"foo", b"bar").unwrap();

//   let adapter = store.close();
//   let mut store = KVStore::open(adapter, false).unwrap();

//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 3);
//   assert_eq!(&buf[..n as usize], b"bar");
// }

// #[test]
// fn test_reinsert() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert(b"foo", b"Lorem").unwrap();
//   store.insert(b"foo", b"ipsum").unwrap();
//   store.insert(b"foo", b"dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua").unwrap();
//   let (_, len, cap) = store.load(b"foo", &mut buf).unwrap();
//   assert_eq!(len, 110);
//   assert_eq!(cap, 110);

//   store.insert(b"foo", b"bar").unwrap();
//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 3);
//   assert_eq!(&buf[..n as usize], b"bar");
// }

// #[test]
// fn test_insert_alloc() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store
//     .insert_with_capacity(b"foo", b"bar", Some(16))
//     .unwrap();
//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 16);
//   assert_eq!(&buf[..n as usize], b"bar");
// }

// #[test]
// fn test_patch() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store
//     .insert_with_capacity(b"foo", b"aar", Some(16))
//     .unwrap();
//   let (len, cap) = store.patch(b"foo", 0, b"b").unwrap();

//   assert_eq!(len, 3);
//   assert_eq!(cap, 16);

//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 16);
//   assert_eq!(&buf[..n as usize], b"bar");
// }

// #[test]
// fn test_append() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 7);
//   assert_eq!(&buf[..n as usize], b"bar");

//   let (len, cap) = store.patch(b"foo", 3, b" baz").unwrap();
//   assert_eq!(len, 7);
//   assert_eq!(cap, 7);

//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 7);
//   assert_eq!(len, 7);
//   assert_eq!(cap, 7);
//   assert_eq!(&buf[..n as usize], b"bar baz");
// }

// #[test]
// fn test_patch_with_hole() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
//   store.load(b"foo", &mut buf).unwrap();
//   let err = store.patch(b"foo", 4, b"baz").unwrap_err();

//   assert_eq!(err, Error::InvalidPatchOffset);
// }

// #[test]
// fn test_patch_realloc() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
//   let (len, cap) = store.patch(b"foo", 3, b" baz bar").unwrap();
//   assert_eq!(len, 11);
//   assert_eq!(cap, 11);

//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 11);
//   assert_eq!(len, 11);
//   assert_eq!(cap, 11);
//   assert_eq!(&buf[..n as usize], b"bar baz bar");
// }

// #[test]
// fn test_append_realloc() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert_with_capacity(b"foo", b"bar", Some(7)).unwrap();
//   let (len, cap) = store.append(b"foo", b" baz bar").unwrap();
//   assert_eq!(len, 11);
//   assert_eq!(cap, 11);

//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 11);
//   assert_eq!(len, 11);
//   assert_eq!(cap, 11);
//   assert_eq!(&buf[..n as usize], b"bar baz bar");
// }

// #[test]
// fn test_remove() {
//   let mut buf = [0; 255];
//   let mut store = KVStore::open(MemAdapter::new(), true).unwrap();

//   store.insert(b"foo", b"bar").unwrap();
//   store.remove(b"foo").unwrap();
//   let err = store.load(b"foo", &mut buf).unwrap_err();
//   assert_eq!(err, Error::KeyNofFound);

//   store
//     .insert_with_capacity(b"foo", b"bar", Some(16))
//     .unwrap();
//   let (n, len, cap) = store.load(b"foo", &mut buf).unwrap();

//   assert_eq!(n, 3);
//   assert_eq!(len, 3);
//   assert_eq!(cap, 16);
//   assert_eq!(&buf[..n as usize], b"bar");
// }
