use core::cmp::{Eq, Ord, Ordering, PartialEq};

#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Slot {
    start: usize,
    end: usize,
}

impl Slot {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

impl Ord for Slot {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size().cmp(&other.size())
    }
}

impl PartialOrd for Slot {
    fn partial_cmp(&self, other: &Slot) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Slot {
    fn eq(&self, other: &Slot) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Slot {}

pub struct Alloc<const SLOTS: usize> {
    pub(crate) slots: [Slot; SLOTS],
}

impl<const SLOTS: usize> Alloc<SLOTS> {
    pub fn new(start: usize, space: usize) -> Self {
        let mut slots = [Slot::default(); SLOTS];
        slots[0] = Slot::new(start, space + start);
        Self { slots }
    }

    pub fn alloc(&mut self, size: usize, addr: Option<usize>) -> Option<usize> {
        if let Some(addr) = addr {
            match self
                .slots
                .iter_mut()
                .find(|s| addr >= s.start && addr < s.end && s.size() - (addr - s.start) >= size)
            {
                Some(slot) if slot.start == addr => {
                    slot.start += size;
                    Some(addr)
                }
                Some(slot) => {
                    let slot_end = slot.end;
                    slot.end = addr;
                    if let Some(unused_slot) = self.slots.iter_mut().find(|s| s.size() == 0) {
                        unused_slot.start = addr + size;
                        unused_slot.end = slot_end;
                    } else {
                        return None;
                    };
                    Some(addr)
                }
                _ => None,
            }
        } else {
            let slot = self.slots.iter_mut().filter(|s| s.size() >= size).max()?;
            let start = slot.start;
            slot.start += size;
            Some(start)
        }
    }

    pub fn free(&mut self, addr: usize, size: usize) {
        let slot_end = addr + size;
        if let Some(slot) = self.slots.iter_mut().find(|s| s.end == addr) {
            slot.end += size;
        } else if let Some(slot) = self.slots.iter_mut().find(|s| s.start == slot_end) {
            slot.start = addr;
        } else if let Some(slot) = self.slots.iter_mut().find(|s| s.size() == 0) {
            slot.start = addr;
            slot.end = slot_end;
        }
    }
}
