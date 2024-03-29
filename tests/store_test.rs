use std::collections::HashSet;

use kvs::adapters::ram::*;
use kvs::{Grasshopper, KVStore, StoreConfig};

const KEY_COLLISION_HASH: u16 = 58263;

const KEY_COLLISIONS: [&str; 16] = [
    "/bin/charlotte/big/great/year.jpg",
    "/bin/group/place/public.txt",
    "/bin/oliver/fact/year/place.txt",
    "/etc/able/woman/mia.mp4",
    "/etc/big/week.jpg",
    "/home/able/year.rar",
    "/home/early/group.jpg",
    "/isabella/time.mp4",
    "/sbin/young/eye/problem.xls",
    "/thing/charlotte.flv",
    "/tmp/amelia/emma/good.tar",
    "/usr/elijah/right.tar",
    "/usr/emma.rar",
    "/usr/olivia/different.jpg",
    "/var/aria/high.zip",
    "/var/life/time/oliver.tar",
];

#[test]
fn test_collisions() {
    for key in KEY_COLLISIONS.iter() {
        let hash = Grasshopper::<{ tiny::BUCKETS }>::new(tiny::MAX_HOPS, 0, key.as_bytes()).hash();
        assert_eq!(KEY_COLLISION_HASH, hash);
    }
}

mod tiny {
    use crate::*;

    pub const MAGIC: u32 = 0x796e6974;
    pub const STORE_SIZE: usize = 1024;
    pub const BUCKETS: usize = 32;
    pub const SLOTS: usize = 8;
    pub const MAX_HOPS: usize = 32;

    pub type Store = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

    pub fn create_store() -> Store {
        Store::open(
            MemoryAdapter::default(),
            StoreConfig::new(MAGIC, MAX_HOPS),
            true,
        )
        .unwrap()
    }
}

#[test]
fn test_create_store() {
    tiny::create_store();
}

#[test]
fn test_create_invalid_store() {
    type TooSmallStore = KVStore<MemoryAdapter<2>, { tiny::BUCKETS }, { tiny::SLOTS }>;

    let store = TooSmallStore::open(
        MemoryAdapter::default(),
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS),
        true,
    );
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::AdapterError(()));
    }
}

#[test]
fn test_reopen_store() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(
        adapter,
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS),
        false,
    );
    assert!(store.is_ok());
}

#[test]
fn test_reopen_store_with_invalid_magic() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(
        adapter,
        StoreConfig::new(tiny::MAGIC + 1, tiny::MAX_HOPS),
        false,
    );
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::StoreNotFound);
    }
}

#[test]
fn test_reopen_store_with_invalid_nonce() {
    let adapter = tiny::create_store().close();

    let store = tiny::Store::open(
        adapter,
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS).nonce(1),
        false,
    );
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::InvalidNonce);
    }
}

#[test]
fn test_reopen_store_with_invalid_buckets() {
    let adapter = tiny::create_store().close();

    type WrongCapacityStore =
        KVStore<MemoryAdapter<{ tiny::STORE_SIZE }>, { tiny::BUCKETS * 2 }, { tiny::SLOTS }>;

    let store = WrongCapacityStore::open(
        adapter,
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS),
        false,
    );
    assert!(store.is_err());
    if let Err(err) = store {
        assert_eq!(err, kvs::Error::InvalidCapacity);
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
    assert_eq!(check.len(), 3);

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
fn test_list_keys_with_prefix() {
    let mut check = HashSet::new();
    check.insert("/root/foo");
    check.insert("/root/bar");
    check.insert("/root/baz");
    check.insert("/etc/foo");
    assert_eq!(check.len(), 4);

    let mut store = tiny::create_store();

    store.insert(b"/root/foo", b"bar").unwrap();
    store.insert(b"/root/bar", b"barbaz").unwrap();
    store.insert(b"/root/baz", b"foobarbaz").unwrap();
    store.insert(b"/etc/foo", b"baz").unwrap();

    for key_ref in store.keys_with_prefix(b"/root") {
        let key = core::str::from_utf8(key_ref.key()).unwrap();
        check.remove(key);
    }

    assert_eq!(check.len(), 1);
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
fn test_exists() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    assert_eq!(store.exists(b"foo").unwrap(), true);
    assert_eq!(store.exists(b"bar").unwrap(), false);
}

#[test]
fn test_load_slice() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let val = store.load_slice(b"foo", &mut scratch).unwrap();
    assert_eq!(val, b"bar");

    let mut scratch = [0; 2];
    let val = store.load_slice(b"foo", &mut scratch).unwrap();
    assert_eq!(val, b"ba");
}

#[test]
fn test_load_str() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let val = store.load_str(b"foo", &mut scratch).unwrap();
    assert_eq!(val, "bar");

    let mut scratch = [0; 2];
    let val = store.load_str(b"foo", &mut scratch).unwrap();
    assert_eq!(val, "ba");
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
    let mut store = tiny::Store::open(
        adapter,
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS),
        false,
    )
    .unwrap();

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
fn test_alloc() {
    let mut store = tiny::create_store();
    store.alloc(b"foo", 9, None).unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 9);
    assert_eq!(bucket.record_len(), 12);
    assert_eq!(&scratch[..bucket.val_len()], &[0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn test_alloc_fill() {
    let mut store = tiny::create_store();
    store.alloc(b"foo", 9, Some(b'x')).unwrap();

    let mut scratch = [0; 16];
    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 9);
    assert_eq!(bucket.record_len(), 12);
    assert_eq!(&scratch[..bucket.val_len()], b"xxxxxxxxx");
}

#[test]
fn test_patch() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"Bar").unwrap();

    let bucket = store.patch(b"foo", 0, b"b").unwrap();
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

    let err = store.patch(b"foo", 4, b"baz").unwrap_err();
    assert_eq!(err, kvs::Error::InvalidPatchOffset);
}

#[test]
fn test_patch_no_space() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();
    store.insert(b"bar", b"baz").unwrap();

    let err = store.append(b"foo", b"baz").unwrap_err();
    assert_eq!(err, kvs::Error::ValueOverflow);
}

#[test]
fn test_patch_resize() {
    let mut store = tiny::create_store();
    store.insert(b"foo", b"bar").unwrap();

    let mut scratch = [0; 16];
    let bucket = store.patch(b"foo", 3, b" baz bar").unwrap();
    assert_eq!(bucket.val_len(), 11);

    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.val_len(), 11);
    assert_eq!(bucket.record_len(), 14);
    assert_eq!(&scratch[..bucket.val_len()], b"bar baz bar");
}

#[test]
fn test_reset() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"bar").unwrap();
    store.reset().unwrap();

    let mut scratch = [0; 16];
    let err = store.load(b"foo", &mut scratch).unwrap_err();
    assert_eq!(err, kvs::Error::KeyNotFound);
}

#[test]
fn test_remove() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"bar").unwrap();
    store.remove(b"foo").unwrap();

    let mut scratch = [0; 16];
    let err = store.load(b"foo", &mut scratch).unwrap_err();
    assert_eq!(err, kvs::Error::KeyNotFound);

    store.insert(b"foo", b"bar").unwrap();

    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_erase() {
    let mut store = tiny::create_store();

    store.insert(b"foo", b"bar").unwrap();
    store.erase(b"foo", 0).unwrap();

    let mut scratch = [0; 16];
    let err = store.load(b"foo", &mut scratch).unwrap_err();
    assert_eq!(err, kvs::Error::KeyNotFound);

    store.insert(b"foo", b"bar").unwrap();

    let bucket = store.load(b"foo", &mut scratch).unwrap();
    assert_eq!(bucket.key_len(), 3);
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_hash_collision() {
    let mut store = tiny::create_store();
    store.insert(KEY_COLLISIONS[0].as_bytes(), b"foo").unwrap();
    store.insert(KEY_COLLISIONS[1].as_bytes(), b"bar").unwrap();

    let mut scratch = [0; 16];
    let bucket = store
        .load(KEY_COLLISIONS[0].as_bytes(), &mut scratch)
        .unwrap();
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"foo");

    let bucket = store
        .load(KEY_COLLISIONS[1].as_bytes(), &mut scratch)
        .unwrap();
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_hash_collision_broken_chain() {
    let mut store = tiny::create_store();
    store.insert(KEY_COLLISIONS[0].as_bytes(), b"foo").unwrap();
    store.insert(KEY_COLLISIONS[1].as_bytes(), b"bar").unwrap();
    store.remove(KEY_COLLISIONS[0].as_bytes()).unwrap();

    let mut scratch = [0; 16];
    let err = store
        .load(KEY_COLLISIONS[0].as_bytes(), &mut scratch)
        .unwrap_err();
    assert_eq!(err, kvs::Error::KeyNotFound);

    let bucket = store
        .load(KEY_COLLISIONS[1].as_bytes(), &mut scratch)
        .unwrap();
    assert_eq!(bucket.val_len(), 3);
    assert_eq!(&scratch[..bucket.val_len()], b"bar");
}

#[test]
fn test_hash_collisions() {
    let mut store = tiny::create_store();
    for (idx, key) in KEY_COLLISIONS.iter().enumerate() {
        store.insert(key.as_bytes(), &[idx as u8]).unwrap();
    }

    let mut scratch = [0; 16];
    for (idx, key) in KEY_COLLISIONS.iter().enumerate() {
        let bucket = store.load(key.as_bytes(), &mut scratch).unwrap();
        assert_eq!(bucket.val_len(), 1);
        assert_eq!(&scratch[..bucket.val_len()], &[idx as u8]);
    }
}

#[test]
fn test_compatibility() {
    let data = include_bytes!("./tiny.db");

    let mut store = tiny::Store::open(
        MemoryAdapter::new(*data),
        StoreConfig::new(tiny::MAGIC, tiny::MAX_HOPS).nonce(34),
        false,
    )
    .unwrap();

    let mut scratch = [0; 64];
    for key in KEY_COLLISIONS.iter() {
        let bucket = store.load(key.as_bytes(), &mut scratch).unwrap();
        assert_eq!(bucket.val_len(), key.len());
        assert_eq!(&scratch[..bucket.val_len()], key.as_bytes());
    }
}
