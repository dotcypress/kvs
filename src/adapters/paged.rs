use crate::adapters::*;

pub struct PagedAdapter<A, const PAGE_SIZE: usize>
where
    A: StoreAdapter,
{
    inner: A,
    offset: usize,
}

impl<A, const PAGE_SIZE: usize> PagedAdapter<A, PAGE_SIZE>
where
    A: StoreAdapter,
{
    pub fn new(inner: A, offset: usize) -> Self {
        Self { inner, offset }
    }
}

impl<A, const PAGE_SIZE: usize> StoreAdapter for PagedAdapter<A, PAGE_SIZE>
where
    A: StoreAdapter,
{
    type Error = A::Error;

    fn max_address(&self) -> Address {
        self.inner.max_address()
    }

    fn read(&mut self, addr: Address, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read(addr + self.offset, buf)
    }

    fn write(&mut self, addr: Address, data: &[u8]) -> Result<(), Self::Error> {
        let addr = addr + self.offset;
        let page_offset = addr % PAGE_SIZE;
        if page_offset + data.len() <= PAGE_SIZE {
            return self.inner.write(addr, data);
        }

        let mut offset = 0;
        let mut chunk = PAGE_SIZE - page_offset;
        while chunk > 0 {
            self.inner
                .write(addr + offset, &data[offset..(offset + chunk)])?;
            offset += chunk;
            chunk = usize::min(PAGE_SIZE, data.len() - offset);
        }

        Ok(())
    }
}
