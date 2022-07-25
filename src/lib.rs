#![no_std]

use adapters::StoreAdapter;
use modular_bitfield::prelude::*;

mod alloc;
mod grasshopper;
mod store;

pub mod adapters;

pub use alloc::*;
pub use grasshopper::*;
pub use store::*;

pub const MAX_KEY_LEN: usize = 256;
pub const MAX_VALUE_LEN: usize = 64 * 1024;

pub type Address = usize;

#[derive(Debug)]
pub struct Bucket {
    index: usize,
    raw: RawBucket,
}

impl Bucket {
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn address(&self) -> Address {
        self.raw.address() as Address
    }

    pub fn key_len(&self) -> usize {
        self.raw.key_len() as usize
    }

    pub fn val_len(&self) -> usize {
        self.raw.val_len() as usize
    }

    pub fn record_len(&self) -> usize {
        self.key_len() + self.val_len()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error<E> {
    AdapterError(E),
    IndexOverflow,
    InvalidCapacity,
    InvalidNonce,
    InvalidPatchOffset,
    KeyNotFound,
    ReadOnlyStore,
    StoreNotFound,
    StoreOverflow,
    ValueOverflow,
    #[cfg(feature = "crud")]
    KeyAlreadyExists,
}

#[bitfield]
pub(crate) struct StoreHeader {
    magic: B32,
    nonce: B16,
    buckets: B16,
}

#[bitfield]
#[derive(Default, Debug, Clone)]
pub(crate) struct RawBucket {
    val_len: B16,
    key_len: B8,
    address: B24,
    hash: B16,
}

pub struct KeysIterator<'a, A, const BUCKETS: usize, const SLOTS: usize>
where
    A: StoreAdapter,
{
    store: &'a mut KVStore<A, BUCKETS, SLOTS>,
    cursor: usize,
}

impl<'a, A, const BUCKETS: usize, const SLOTS: usize> KeysIterator<'a, A, BUCKETS, SLOTS>
where
    A: StoreAdapter,
{
    pub fn new(store: &'a mut KVStore<A, BUCKETS, SLOTS>) -> Self {
        Self { store, cursor: 0 }
    }
}

impl<'a, A, const BUCKETS: usize, const SLOTS: usize> Iterator
    for KeysIterator<'a, A, BUCKETS, SLOTS>
where
    A: StoreAdapter,
{
    type Item = KeyReference;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.cursor >= BUCKETS {
                return None;
            }

            let raw = self.store.load_bucket(self.cursor).unwrap_or_default();
            let index = self.cursor;
            self.cursor += 1;

            let key_len = raw.key_len() as usize;
            if key_len > 0 {
                let val_len = raw.val_len() as usize;
                let bucket = Bucket { index, raw };
                let address = bucket.raw.address() as Address;
                let mut scratch = [0; MAX_KEY_LEN];

                self.store
                    .adapter()
                    .read(address, &mut scratch[..key_len])
                    .ok();

                return Some(KeyReference {
                    key_len,
                    val_len,
                    scratch,
                });
            }
        }
    }
}

pub struct KeyReference {
    key_len: usize,
    val_len: usize,
    scratch: [u8; MAX_KEY_LEN],
}

impl KeyReference {
    pub fn key(&self) -> &[u8] {
        &self.scratch[..self.key_len]
    }

    pub fn val_len(&self) -> usize {
        self.val_len
    }
}
