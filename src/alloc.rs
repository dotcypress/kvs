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
    slots: [Slot; SLOTS],
    #[allow(dead_code)]
    space: usize,
}

impl<const SLOTS: usize> Alloc<SLOTS> {
    pub fn new(start: usize, space: usize) -> Self {
        let mut slots = [Slot::default(); SLOTS];
        slots[0] = Slot::new(start, space  + start);
        Self { space, slots }
    }

    pub fn alloc(&mut self, size: usize, addr: Option<usize>) -> Option<usize> {
        if let Some(addr) = addr {
            match self
                .slots
                .iter_mut()
                .find(|h| addr >= h.start && addr < h.end && h.size() - (addr - h.start) >= size)
            {
                Some(hole) if hole.start == addr => {
                    hole.start += size;
                    Some(addr)
                }
                Some(hole) => {
                    let hole_end = hole.end;
                    hole.end = addr;
                    if let Some(unused_hole) = self.slots.iter_mut().find(|h| h.size() == 0) {
                        unused_hole.start = addr + size;
                        unused_hole.end = hole_end;
                    } else {
                        return None;
                    };
                    Some(addr)
                }
                _ => None,
            }
        } else {
            let hole = self.slots.iter_mut().filter(|h| h.size() >= size).max()?;
            let start = hole.start;
            hole.start += size;
            Some(start)
        }
    }

    pub fn free(&mut self, addr: usize, size: usize) {
        let hole_end = addr + size;
        if let Some(hole) = self.slots.iter_mut().find(|h| h.end == addr) {
            hole.end += size;
        } else if let Some(hole) = self.slots.iter_mut().find(|h| h.start == hole_end) {
            hole.start = addr;
        } else if let Some(slot) = self.slots.iter_mut().find(|h| h.size() == 0) {
            slot.start = addr;
            slot.end = hole_end;
        }
    }
}
