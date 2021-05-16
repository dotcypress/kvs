use std::collections::HashSet;

use kvs::{KVStore, MemoryAdapter};

mod tiny {
    use crate::*;

    pub const MAGIC: u32 = 0x4b1d;
    pub const STORE_SIZE: usize = 1024;
    pub const BUCKETS: usize = 64;
    pub const SLOTS: usize = 8;

    pub type Store = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

    pub fn create_store() -> Store {
        Store::create(MemoryAdapter::default(), MAGIC).unwrap()
    }
}

#[test]
fn test_create_store() {
    tiny::create_store();
}

#[test]
fn test_create_invalid_store() {
    type TooSmallStore = KVStore<MemoryAdapter<2>, { tiny::BUCKETS }, { tiny::SLOTS }>;

    let store = TooSmallStore::create(MemoryAdapter::default(), tiny::MAGIC);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::AdapterError(()));
    }
}

#[test]
fn test_reopen_store() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(adapter, tiny::MAGIC);
    assert!(store.is_ok());
}

#[test]
fn test_reopen_store_with_invalid_magic() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(adapter, tiny::MAGIC + 1);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::StoreNotFound);
    }
}

#[test]
fn test_reopen_store_with_invalid_buckets() {
    let adapter = tiny::create_store().close();

    type WrongCapacityStore =
        KVStore<MemoryAdapter<{ tiny::STORE_SIZE }>, { tiny::BUCKETS * 2 }, { tiny::SLOTS }>;

    let store = WrongCapacityStore::open(adapter, tiny::MAGIC);
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::InvalidBuckets);
    }
}

#[test]
fn test_insert() {
    let mut store = tiny::create_store();
    let bucket = store.insert(b"foo", b"bar").unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
}

#[test]
fn test_list_keys() {
    let mut check = HashSet::new();
    check.insert("foo");
    check.insert("bar");
    check.insert("baz");

    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();
    store.insert(b"bar", b"barbaz").unwrap();
    store.insert(b"baz", b"foobarbaz").unwrap();

    for key_ref in store.keys() {
        let key = core::str::from_utf8(key_ref.key()).unwrap();
        check.remove(key);
    }

    assert_eq!(check.len(), 0);
}

#[test]
fn test_load() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_load_rewrited() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"Ba").unwrap();
    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_reopen() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"bar").unwrap();

    let adapter = store.close();
    let mut store = tiny::Store::open(adapter, tiny::MAGIC).unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_rewrite() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"Lorem").unwrap();
    store.insert(b"foo", b"ipsum").unwrap();
    store.insert(b"foo", b"dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua").unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.val_len(), 110);

    store.insert(b"foo", b"bar").unwrap();
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_patch() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"Bar").unwrap();
    let bucket = store.patch(b"foo", b"b", 0).unwrap();
    assert_eq!(bucket.key_len(), 3);

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(bucket.record_len(), 6);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_append() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");

    let bucket = store.append(b"foo", b" baz").unwrap();
    assert_eq!(bucket.val_len(), 7);

    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.val_len(), 7);
    assert_eq!(&scratch[..bucket.val_len()], b"bar baz");
}

#[test]
fn test_patch_with_hole() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let err = store.patch(b"foo", b"baz", 4).unwrap_err();
    assert_eq!(err, kvs::Error::InvalidPatchOffset);
}

#[test]
fn test_patch_resize() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let bucket = store.patch(b"foo", b" baz bar", 3).unwrap();
    assert_eq!(bucket.val_len(), 11);

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.val_len(), 11);
    assert_eq!(bucket.record_len(), 14);
    assert_eq!(&scratch[..bucket.val_len()], b"bar baz bar");
}

#[test]
fn test_remove() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    store.remove(b"foo").unwrap();

    let mut scratch = [0; 16];
    let err = store.load(b"foo", &mut scratch).unwrap_err();
    assert_eq!(err, kvs::Error::KeyNofFound);

    store.insert(b"foo", b"bar").unwrap();
    let bucket = store.load(b"foo", &mut scratch).unwrap();

    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}
