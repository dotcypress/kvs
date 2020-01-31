#[derive(Debug, Default, Copy, Clone)]
pub struct Hole {
    pub idx: usize,
    pub from: u16,
    pub to: u16,
}

impl core::cmp::Ord for Hole {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.size().cmp(&other.size())
    }
}

impl core::cmp::PartialOrd for Hole {
    fn partial_cmp(&self, other: &Hole) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::cmp::PartialEq for Hole {
    fn eq(&self, other: &Hole) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl core::cmp::Eq for Hole {}

impl Hole {
    pub fn size(self) -> u16 {
        self.to - self.from
    }
}
