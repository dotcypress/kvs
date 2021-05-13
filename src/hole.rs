use core::cmp::{Eq, Ord, Ordering, PartialEq};

#[derive(Debug, Default, Copy, Clone)]
pub struct Hole {
    pub start: usize,
    pub end: usize,
}

impl Hole {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn reset(&mut self) {
        self.end = 0;
        self.start = 0;
    }

    pub fn size(&self) -> usize {
        self.end - self.start
    }
}

impl Ord for Hole {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size().cmp(&other.size())
    }
}

impl PartialOrd for Hole {
    fn partial_cmp(&self, other: &Hole) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Hole {
    fn eq(&self, other: &Hole) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Hole {}
