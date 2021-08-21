#![no_std]

use modular_bitfield::prelude::*;

mod alloc;
mod grasshopper;
mod store;

pub mod adapters;

pub use alloc::*;
pub use grasshopper::*;
pub use store::*;

pub const MAX_KEY_LEN: usize = 128;
pub const MAX_VALUE_LEN: usize = 32 * 1024;

const BUCKET_BATCH_SIZE: usize = 32;

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
    KeyNofFound,
    ReadOnlyStore,
    StoreNotFound,
    StoreOverflow,
    ValueOverflow,
}

#[bitfield]
pub(crate) struct RawStoreHeader {
    magic: B32,
    nonce: B16,
    buckets: B16,
}

#[bitfield]
#[derive(Default, Debug, Clone)]
pub(crate) struct RawBucket {
    in_use: bool,
    val_len: B15,
    key_len: B8,
    address: B24,
    hash: B16,
}
