#![cfg(feature = "serde")]

use kvs::adapters::ram::*;
use kvs::{KVStore, StoreConfig};

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
enum SensorMode {
    LowPower,
    Normal,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
struct SensorConfig {
    offset: usize,
    prescaler: u8,
    mode: SensorMode,
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

const TEST_ENTRY: SensorConfig = SensorConfig {
    offset: 100500,
    prescaler: 42,
    mode: SensorMode::LowPower,
};

#[test]
fn test_create_store() {
    tiny::create_store();
}

#[test]
fn test_create() {
    let mut store = tiny::create_store();

    let bucket = store
        .insert_object::<SensorConfig, 16>(b"sensor/1", &TEST_ENTRY)
        .unwrap();
    assert_eq!(bucket.key_len(), 8);
    assert_eq!(bucket.val_len(), 5);
    assert_eq!(bucket.record_len(), 13);
}

#[test]
fn test_read() {
    let mut store = tiny::create_store();
    store
        .insert_object::<SensorConfig, 16>(b"sensor/1", &TEST_ENTRY)
        .unwrap();
    let entry = store.load_object::<SensorConfig, 16>(b"sensor/1").unwrap();
    assert_eq!(entry, TEST_ENTRY);
}

#[test]
fn test_update() {
    let mut store = tiny::create_store();
    store
        .insert_object::<SensorConfig, 16>(b"sensor/1", &TEST_ENTRY)
        .unwrap();
    let mut entry = store.load_object::<SensorConfig, 16>(b"sensor/1").unwrap();
    entry.mode = SensorMode::Normal;
    store
        .patch_object::<SensorConfig, 16>(b"sensor/1", &entry)
        .unwrap();

    let entry = store.load_object::<SensorConfig, 16>(b"sensor/1").unwrap();
    assert_eq!(entry.mode, SensorMode::Normal);
}

#[test]
fn test_delete() {
    let mut store = tiny::create_store();
    store
        .insert_object::<SensorConfig, 16>(b"sensor/1", &TEST_ENTRY)
        .unwrap();

    store.remove(b"sensor/1").unwrap();
    let res = store.load_object::<SensorConfig, 16>(b"sensor/1");
    assert_eq!(res, Err(kvs::Error::KeyNotFound));
}
