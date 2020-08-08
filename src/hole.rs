use core::cmp::{Eq, Ord, Ordering, PartialEq};

#[derive(Debug, Default, Copy, Clone)]
pub struct Hole {
    pub from: u32,
    pub to: u32,
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
        self.from == other.from && self.to == other.to
    }
}

impl Eq for Hole {}

impl Hole {
    pub fn size(&self) -> u32 {
        self.to - self.from
    }

    pub fn reset(&mut self) {
        self.to = 0;
        self.from = 0;
    }
}
