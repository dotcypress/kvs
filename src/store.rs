use crate::adapters::*;
use crate::*;
use core::mem::size_of;

#[cfg(feature = "crud")]
use heapless::Vec;
#[cfg(feature = "crud")]
use postcard::{from_bytes, to_vec};
#[cfg(feature = "crud")]
use serde::{de::DeserializeOwned, Serialize};

pub struct StoreConfig {
    magic: u32,
    nonce: u16,
    max_hops: usize,
}

impl StoreConfig {
    pub fn new(magic: u32, max_hops: usize) -> Self {
        Self {
            magic,
            max_hops,
            nonce: 0,
        }
    }

    pub fn nonce(self, nonce: u16) -> Self {
        let mut res = self;
        res.nonce = nonce;
        res
    }
}

pub struct KVStore<A, const BUCKETS: usize, const SLOTS: usize>
where
    A: StoreAdapter,
{
    adapter: A,
    cfg: StoreConfig,
    alloc: Option<Alloc<SLOTS>>,
    scratch: [u8; MAX_KEY_LEN],
}

pub type ReadOnlyKVStore<A, const BUCKETS: usize> = KVStore<A, BUCKETS, 0>;

impl<E, A, const BUCKETS: usize, const SLOTS: usize> KVStore<A, BUCKETS, SLOTS>
where
    A: StoreAdapter<Error = E>,
{
    const DATA_START: Address = size_of::<StoreHeader>() + size_of::<RawBucket>() * BUCKETS;

    pub fn open(adapter: A, cfg: StoreConfig, create_new: bool) -> Result<Self, Error<E>> {
        let mut adapter = adapter;
        match Self::load_header(&mut adapter, cfg.magic, cfg.nonce) {
            Ok(_) => Ok(Self {
                alloc: None,
                scratch: [0; MAX_KEY_LEN],
                adapter,
                cfg,
            }),
            Err(Error::StoreNotFound) if create_new => Self::bootstrap(adapter, cfg),
            Err(err) => Err(err),
        }
    }

    pub fn bootstrap(adapter: A, cfg: StoreConfig) -> Result<Self, Error<E>> {
        let mut res = Self {
            alloc: None,
            scratch: [0; MAX_KEY_LEN],
            adapter,
            cfg,
        };
        res.reset()?;
        Ok(res)
    }

    pub fn adapter(&mut self) -> &mut A {
        &mut self.adapter
    }

    pub fn close(self) -> A {
        self.adapter
    }

    pub fn reset(&mut self) -> Result<(), Error<E>> {
        const ERASE_BATCH_SIZE: usize = 32;

        let zeroes = [0; size_of::<RawBucket>() * ERASE_BATCH_SIZE];
        let mut offset = size_of::<StoreHeader>();
        let mut buckets = BUCKETS;
        while buckets > 0 {
            let batch = usize::min(buckets, ERASE_BATCH_SIZE);
            buckets -= batch;

            let chunk = batch * size_of::<RawBucket>();
            self.adapter
                .write(offset, &zeroes[..chunk])
                .map_err(Error::AdapterError)?;
            offset += chunk;
        }

        let header = StoreHeader::new()
            .with_magic(self.cfg.magic)
            .with_nonce(self.cfg.nonce)
            .with_buckets(BUCKETS as u16);

        self.adapter
            .write(0, &header.into_bytes())
            .map_err(Error::AdapterError)?;

        Ok(())
    }

    pub fn alloc(
        &mut self,
        key: &[u8],
        val_len: usize,
        fill_with: Option<u8>,
    ) -> Result<Bucket, Error<E>> {
        let bucket = self.alloc_bucket(key, val_len)?;
        if let Some(fill_with) = fill_with {
            self.erase_bucket_content(&bucket, fill_with)?;
        }
        self.adapter
            .write(bucket.address(), key)
            .map_err(Error::AdapterError)?;
        Ok(bucket)
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) -> Result<Bucket, Error<E>> {
        let bucket = self.alloc_bucket(key, val.len())?;

        self.adapter
            .write(bucket.address(), key)
            .map_err(Error::AdapterError)?;

        self.adapter
            .write(bucket.address() + key.len(), val)
            .map_err(Error::AdapterError)?;
        Ok(bucket)
    }

    pub fn append(&mut self, key: &[u8], val: &[u8]) -> Result<Bucket, Error<E>> {
        assert!(
            !key.is_empty()
                && key.len() <= MAX_KEY_LEN
                && !val.is_empty()
                && val.len() <= MAX_VALUE_LEN
        );
        if SLOTS == 0 {
            return Err(Error::ReadOnlyStore);
        }

        let bucket = self.lookup(key)?;
        assert!(bucket.val_len() + val.len() <= MAX_VALUE_LEN);
        let offset = bucket.val_len();
        self.patch_value(bucket, offset, val)
    }

    pub fn patch(&mut self, key: &[u8], offset: usize, patch: &[u8]) -> Result<Bucket, Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN && !patch.is_empty());

        let bucket = self.lookup(key)?;
        assert!(bucket.val_len() + patch.len() <= MAX_VALUE_LEN);

        if offset > bucket.val_len() {
            return Err(Error::InvalidPatchOffset);
        }
        self.patch_value(bucket, offset, patch)
    }

    pub fn load(&mut self, key: &[u8], buf: &mut [u8]) -> Result<Bucket, Error<E>> {
        self.load_at(key, buf, 0)
    }

    pub fn load_at(
        &mut self,
        key: &[u8],
        buf: &mut [u8],
        offset: usize,
    ) -> Result<Bucket, Error<E>> {
        assert!(
            !key.is_empty()
                && key.len() <= MAX_KEY_LEN
                && !buf.is_empty()
                && (offset + buf.len()) <= MAX_VALUE_LEN
        );

        let bucket = self.lookup(key)?;
        let addr = bucket.address() + bucket.key_len() + offset;
        let size = usize::min(buf.len(), bucket.val_len());
        self.adapter
            .read(addr, &mut buf[..size])
            .map_err(Error::AdapterError)?;
        Ok(bucket)
    }

    pub fn erase(&mut self, key: &[u8], fill_with: u8) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        match self.lookup(key) {
            Ok(bucket) => {
                self.erase_bucket_content(&bucket, fill_with)?;
                let addr = size_of::<StoreHeader>() + size_of::<RawBucket>() * bucket.index();
                self.adapter
                    .write(addr, &RawBucket::new().into_bytes())
                    .map_err(Error::AdapterError)?;
                if SLOTS > 0 {
                    self.get_alloc()?
                        .free(bucket.address(), bucket.record_len());
                }
                Ok(())
            }
            Err(Error::KeyNotFound) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        match self.lookup(key) {
            Ok(bucket) => {
                let addr = size_of::<StoreHeader>() + size_of::<RawBucket>() * bucket.index();
                self.adapter
                    .write(addr, &RawBucket::new().into_bytes())
                    .map_err(Error::AdapterError)?;
                if SLOTS > 0 {
                    self.get_alloc()?
                        .free(bucket.address(), bucket.record_len());
                }
                Ok(())
            }
            Err(Error::KeyNotFound) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn keys(&mut self) -> KeysIterator<'_, A, BUCKETS, SLOTS> {
        KeysIterator::new(self)
    }

    pub fn lookup(&mut self, key: &[u8]) -> Result<Bucket, Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        let hopper = Grasshopper::<BUCKETS>::new(self.cfg.max_hops, self.cfg.nonce, key);
        let hash = hopper.hash();

        for index in hopper {
            let raw = self.load_bucket(index)?;
            if raw.hash() != hash || raw.key_len() as usize != key.len() {
                continue;
            }

            self.adapter
                .read(raw.address() as Address, &mut self.scratch[..key.len()])
                .map_err(Error::AdapterError)?;

            if key != &self.scratch[..key.len()] {
                continue;
            }

            return Ok(Bucket { index, raw });
        }

        Err(Error::KeyNotFound)
    }

    #[cfg(feature = "crud")]
    pub fn create<T: Serialize, const N: usize>(
        &mut self,
        id: &[u8],
        val: &T,
    ) -> Result<Bucket, crate::Error<E>> {
        match self.lookup(id) {
            Err(Error::KeyNotFound) => {
                let val: Vec<u8, N> = to_vec(val).unwrap();
                self.insert(id, &val)
            }
            Err(err) => Err(err),
            Ok(_) => Err(Error::KeyAlreadyExists),
        }
    }

    #[cfg(feature = "crud")]
    pub fn read<T: DeserializeOwned, const N: usize>(
        &mut self,
        id: &[u8],
    ) -> Result<T, crate::Error<E>> {
        let mut buf = [0; N];
        let bucket = self.load(id, &mut buf)?;
        let res = from_bytes(&buf[0..bucket.val_len()]).unwrap();
        Ok(res)
    }

    #[cfg(feature = "crud")]
    pub fn update<T: Serialize, const N: usize>(
        &mut self,
        id: &[u8],
        val: &T,
    ) -> Result<Bucket, crate::Error<E>> {
        let bucket = self.lookup(id)?;
        let patch: Vec<u8, N> = to_vec(val).unwrap();
        self.patch_value(bucket, 0, &patch)
    }

    pub(crate) fn load_bucket(&mut self, bucket_index: usize) -> Result<RawBucket, Error<E>> {
        let offset = size_of::<StoreHeader>() + size_of::<RawBucket>() * bucket_index;
        let mut scratch = [0; size_of::<RawBucket>()];
        self.adapter
            .read(offset, &mut scratch)
            .map_err(Error::AdapterError)?;
        Ok(RawBucket::from_bytes(scratch))
    }

    fn alloc_bucket(&mut self, key: &[u8], val_len: usize) -> Result<Bucket, Error<E>> {
        if SLOTS == 0 {
            return Err(Error::ReadOnlyStore);
        }
        let key_len = key.len();
        assert!(
            key_len <= MAX_KEY_LEN && !key.is_empty() && val_len <= MAX_VALUE_LEN && val_len > 0
        );

        let hopper: Grasshopper<BUCKETS> = Grasshopper::new(BUCKETS, self.cfg.nonce, key);
        let hash = hopper.hash();
        let mut free_bucket: Option<Bucket> = None;

        for index in hopper {
            let mut raw = self.load_bucket(index)?;
            if raw.key_len() > 0 {
                if raw.hash() != hash || raw.key_len() as usize != key_len {
                    continue;
                }

                self.adapter
                    .read(raw.address() as Address, &mut self.scratch[..key_len])
                    .map_err(Error::AdapterError)?;
                if key != &self.scratch[..key_len] {
                    continue;
                }

                let bucket = Bucket { index, raw };
                self.get_alloc()?
                    .free(bucket.address(), bucket.record_len());
                free_bucket = Some(bucket);
                break;
            } else {
                raw.set_hash(hash);
                raw.set_key_len(key_len as u8);

                free_bucket = Some(Bucket { index, raw });
                break;
            }
        }

        let mut bucket = free_bucket.ok_or(Error::IndexOverflow)?;
        let addr = match self.get_alloc()?.alloc(key_len + val_len, None) {
            Some(addr) => addr,
            None => return Err(Error::StoreOverflow),
        };

        bucket.raw.set_key_len(key_len as u8);
        bucket.raw.set_address(addr as u32);
        bucket.raw.set_val_len(val_len as u16);

        self.adapter
            .write(
                size_of::<StoreHeader>() + size_of::<RawBucket>() * bucket.index(),
                &bucket.raw.clone().into_bytes(),
            )
            .map_err(Error::AdapterError)?;

        Ok(bucket)
    }

    fn erase_bucket_content(&mut self, bucket: &Bucket, fill_with: u8) -> Result<(), Error<E>> {
        const FILLER_LEN: usize = 8;
        let filler: [u8; FILLER_LEN] = [fill_with; FILLER_LEN];

        let mut offset = 0;
        let mut chunk = usize::min(FILLER_LEN, bucket.record_len());
        while chunk > 0 {
            self.adapter
                .write(bucket.address() + offset, &filler[..chunk])
                .map_err(Error::AdapterError)?;
            offset += chunk;
            chunk = usize::min(FILLER_LEN, bucket.record_len() - offset);
        }

        Ok(())
    }

    fn patch_value(
        &mut self,
        bucket: Bucket,
        offset: usize,
        patch: &[u8],
    ) -> Result<Bucket, Error<E>> {
        let new_val_len = usize::max(offset + patch.len(), bucket.val_len());
        let mut bucket = bucket;

        if new_val_len > bucket.val_len() {
            self.get_alloc()?
                .alloc(
                    new_val_len - bucket.val_len(),
                    Some(bucket.address() + bucket.record_len()),
                )
                .ok_or(Error::ValueOverflow)?;
            bucket.raw.set_val_len(new_val_len as u16);

            let addr = size_of::<StoreHeader>() + size_of::<RawBucket>() * bucket.index();
            self.adapter
                .write(addr, &bucket.raw.clone().into_bytes())
                .map_err(Error::AdapterError)?;
        }

        let addr = bucket.address() + bucket.key_len() + offset;
        self.adapter
            .write(addr, patch)
            .map_err(Error::AdapterError)?;

        Ok(bucket)
    }

    fn get_alloc(&mut self) -> Result<&mut alloc::Alloc<SLOTS>, Error<E>> {
        if self.alloc.is_none() {
            self.alloc = Some(self.load_index()?);
        }
        Ok(self.alloc.as_mut().unwrap())
    }

    fn load_index(&mut self) -> Result<alloc::Alloc<SLOTS>, Error<E>> {
        const BUCKET_BATCH_SIZE: usize = 32;
        const BUCKET_SIZE: usize = size_of::<RawBucket>();

        let mut buf = [0; BUCKET_SIZE * BUCKET_BATCH_SIZE];
        let mut offset = size_of::<StoreHeader>();
        let mut buckets = BUCKETS;
        let mut alloc = Alloc::<SLOTS>::new(
            Self::DATA_START,
            self.adapter.max_address() - Self::DATA_START,
        );

        while buckets > 0 {
            let batch = usize::min(buckets, BUCKET_BATCH_SIZE);
            let chunk = batch * BUCKET_SIZE;
            self.adapter
                .read(offset, &mut buf[..chunk])
                .map_err(Error::AdapterError)?;

            for bucket_idx in 0..batch {
                let mut scratch = [0; BUCKET_SIZE];
                scratch.copy_from_slice(&buf[bucket_idx * BUCKET_SIZE..][..BUCKET_SIZE]);

                let raw = RawBucket::from_bytes(scratch);
                if raw.key_len() == 0 {
                    continue;
                }
                let addr = raw.address() as Address;
                let size = raw.key_len() as usize + raw.val_len() as usize;
                alloc.alloc(size, Some(addr)).ok_or(Error::StoreOverflow)?;
            }
            offset += chunk;
            buckets -= batch;
        }

        Ok(alloc)
    }

    fn load_header(adapter: &mut A, magic: u32, nonce: u16) -> Result<StoreHeader, Error<E>> {
        let mut buf = [0; size_of::<StoreHeader>()];
        adapter
            .read(0, &mut buf)
            .map_err(Error::AdapterError)
            .map(|_| StoreHeader::from_bytes(buf))
            .and_then(|header| {
                if header.magic() != magic {
                    return Err(Error::StoreNotFound);
                }

                if header.nonce() != nonce {
                    return Err(Error::InvalidNonce);
                }

                if header.buckets() as usize != BUCKETS {
                    return Err(Error::InvalidCapacity);
                }

                Ok(header)
            })
    }
}
