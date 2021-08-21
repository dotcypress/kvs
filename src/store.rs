use crate::*;
use crate::adapters::*;
use core::mem::size_of;

pub struct StoreOptions {
    magic: u32,
    max_hops: usize,
    overwrite: bool,
}

impl StoreOptions {
    pub fn new(magic: u32, max_hops: usize) -> Self {
        Self {
            magic,
            max_hops,
            overwrite: false,
        }
    }

    pub fn overwrite(self, overwrite: bool) -> Self {
        let mut res = self;
        res.overwrite = overwrite;
        res
    }
}

pub struct KVStore<A, const BUCKETS: usize, const SLOTS: usize>
where
    A: StoreAdapter,
{
    adapter: A,
    opts: StoreOptions,
    alloc: Alloc<SLOTS>,
}

pub type ReadOnlyKVStore<A, const BUCKETS: usize> = KVStore<A, BUCKETS, 0>;

impl<E, A, const BUCKETS: usize, const SLOTS: usize> KVStore<A, BUCKETS, SLOTS>
where
    A: StoreAdapter<Error = E>,
{
    const DATA_START: Address = size_of::<RawStoreHeader>() + size_of::<RawBucket>() * BUCKETS;

    fn new(adapter: A, opts: StoreOptions) -> Self {
        Self {
            alloc: Alloc::<SLOTS>::new(Self::DATA_START, adapter.max_address() - Self::DATA_START),
            opts,
            adapter,
        }
    }

    pub fn create(adapter: A, opts: StoreOptions) -> Result<Self, Error<E>> {
        let header = RawStoreHeader::new()
            .with_magic(opts.magic)
            .with_buckets(BUCKETS as u16);

        let mut adapter = adapter;

        let zeroes = [0; size_of::<RawBucket>() * BUCKET_BATCH_SIZE];
        let mut offset = size_of::<RawStoreHeader>();
        let mut buckets = BUCKETS;
        while buckets > 0 {
            let batch = usize::min(buckets, BUCKET_BATCH_SIZE);
            buckets -= batch;

            let chunk = batch * size_of::<RawBucket>();
            adapter
                .write(offset, &zeroes[..chunk])
                .map_err(Error::AdapterError)?;
            offset += chunk;
        }

        adapter
            .write(0, &header.into_bytes())
            .map_err(Error::AdapterError)?;

        Ok(Self::new(adapter, opts))
    }

    pub fn open(adapter: A, opts: StoreOptions) -> Result<Self, Error<E>> {
        let mut adapter = adapter;
        match Self::load_header(&mut adapter, opts.magic) {
            Ok(_) => {
                let mut store = Self::new(adapter, opts);
                store.load_index()?;
                Ok(store)
            }
            Err(Error::StoreNotFound) if opts.overwrite => Self::create(adapter, opts),
            Err(err) => Err(err),
        }
    }

    pub fn adapter_mut(&mut self) -> &mut A {
        &mut self.adapter
    }

    pub fn close(self) -> A {
        self.adapter
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
        self.adapter.read(addr, buf).map_err(Error::AdapterError)?;
        Ok(bucket)
    }

    pub fn erase(&mut self, key: &[u8], fill_with: u8) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        match self.lookup(key) {
            Ok(bucket) => {
                self.erase_bucket_content(&bucket, fill_with)?;
                let addr = size_of::<RawStoreHeader>() + size_of::<RawBucket>() * bucket.index();
                self.adapter
                    .write(addr, &RawBucket::new().into_bytes())
                    .map_err(Error::AdapterError)?;
                if SLOTS > 0 {
                    self.alloc.free(bucket.address(), bucket.record_len());
                }
                Ok(())
            }
            Err(Error::KeyNofFound) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> Result<(), Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        match self.lookup(key) {
            Ok(bucket) => {
                let addr = size_of::<RawStoreHeader>() + size_of::<RawBucket>() * bucket.index();
                self.adapter
                    .write(addr, &RawBucket::new().into_bytes())
                    .map_err(Error::AdapterError)?;
                if SLOTS > 0 {
                    self.alloc.free(bucket.address(), bucket.record_len());
                }
                Ok(())
            }
            Err(Error::KeyNofFound) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn keys(&mut self) -> KeysIterator<'_, A, BUCKETS, SLOTS> {
        KeysIterator::new(self)
    }

    pub fn lookup(&mut self, key: &[u8]) -> Result<Bucket, Error<E>> {
        assert!(!key.is_empty() && key.len() <= MAX_KEY_LEN);

        let hopper: Grasshopper<BUCKETS> = Grasshopper::new(self.opts.max_hops, &key);
        let hash = hopper.hash();

        for index in hopper {
            let raw = self.load_bucket(index)?;
            if !raw.in_use() || raw.hash() != hash || raw.key_len() as usize != key.len() {
                continue;
            }

            let mut scratch = [0; MAX_KEY_LEN];
            self.adapter
                .read(raw.address() as Address, &mut scratch)
                .map_err(Error::AdapterError)?;

            if key != &scratch[..key.len()] {
                continue;
            }

            return Ok(Bucket { index, raw });
        }

        Err(Error::KeyNofFound)
    }

    fn load_bucket(&mut self, bucket_index: usize) -> Result<RawBucket, Error<E>> {
        let offset = size_of::<RawStoreHeader>() + size_of::<RawBucket>() * bucket_index;
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

        let hopper: Grasshopper<BUCKETS> = Grasshopper::new(BUCKETS, &key);
        let hash = hopper.hash();
        let mut free_bucket: Option<Bucket> = None;

        for index in hopper {
            let mut raw = self.load_bucket(index)?;
            if raw.in_use() {
                if raw.hash() != hash || raw.key_len() as usize != key.len() {
                    continue;
                }

                let mut scratch = [0; MAX_KEY_LEN];
                self.adapter
                    .read(raw.address() as Address, &mut scratch[..key.len()])
                    .map_err(Error::AdapterError)?;
                if key != &scratch[..key.len()] {
                    continue;
                }

                let bucket = Bucket { index, raw };
                self.alloc.free(bucket.address(), bucket.record_len());
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
        let addr = match self.alloc.alloc(key_len + val_len, None) {
            Some(addr) => addr,
            None => return Err(Error::StoreOverflow),
        };

        bucket.raw.set_in_use(true);
        bucket.raw.set_address(addr as u32);
        bucket.raw.set_val_len(val_len as u16);

        self.adapter
            .write(
                size_of::<RawStoreHeader>() + size_of::<RawBucket>() * bucket.index(),
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
            self.alloc
                .alloc(
                    new_val_len - bucket.val_len(),
                    Some(bucket.address() + bucket.record_len()),
                )
                .ok_or(Error::ValueOverflow)?;
            bucket.raw.set_val_len(new_val_len as u16);

            let addr = size_of::<RawStoreHeader>() + size_of::<RawBucket>() * bucket.index();
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

    fn load_index(&mut self) -> Result<(), Error<E>> {
        if SLOTS == 0 {
            return Ok(());
        }

        const BUCKET_SIZE: usize = size_of::<RawBucket>();
        let mut buf = [0; BUCKET_SIZE * BUCKET_BATCH_SIZE];
        let mut offset = size_of::<RawStoreHeader>();
        let mut buckets = BUCKETS;

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
                if !raw.in_use() {
                    continue;
                }
                let addr = raw.address() as Address;
                let size = raw.key_len() as usize + raw.val_len() as usize;
                self.alloc
                    .alloc(size, Some(addr))
                    .ok_or(Error::StoreOverflow)?;
            }
            offset += chunk;
            buckets -= batch;
        }

        Ok(())
    }

    fn load_header(adapter: &mut A, magic: u32) -> Result<RawStoreHeader, Error<E>> {
        let mut buf = [0; size_of::<RawStoreHeader>()];
        adapter
            .read(0, &mut buf)
            .map_err(Error::AdapterError)
            .map(|_| RawStoreHeader::from_bytes(buf))
            .and_then(|header| {
                if header.magic() != magic {
                    return Err(Error::StoreNotFound);
                }

                if header.buckets() as usize != BUCKETS {
                    return Err(Error::InvalidCapacity);
                }

                Ok(header)
            })
    }
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

            if raw.in_use() {
                let key_len = raw.key_len() as usize;
                let bucket = Bucket { index, raw };

                let mut key_ref = KeyReference::new(bucket.raw.address() as Address, key_len);
                self.store
                    .adapter
                    .read(key_ref.address, &mut key_ref.scratch[..key_len])
                    .ok();

                return Some(key_ref);
            }
        }
    }
}

pub struct KeyReference {
    address: Address,
    key_len: usize,
    scratch: [u8; MAX_KEY_LEN],
}

impl KeyReference {
    pub fn new(address: Address, key_len: usize) -> Self {
        Self {
            address,
            key_len,
            scratch: [0; MAX_KEY_LEN],
        }
    }

    pub fn key(&self) -> &[u8] {
        &self.scratch[..self.key_len]
    }
}
