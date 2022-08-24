#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct HexCoord {
    pub q: isize,
    pub r: isize,
}

impl HexCoord {
    pub fn new(q: isize, r: isize) -> Self {
        HexCoord { q, r }
    }

    pub fn distance(&self, other: &Self) -> usize {
        let diff = HexCoord {
            q: self.q - other.q,
            r: self.r - other.r,
        };
        return (diff.q.unsigned_abs() + (diff.q + diff.r).unsigned_abs() + diff.r.unsigned_abs())
            / 2;
    }

    pub fn neighbours(&self) -> Vec<Self> {
        vec![
            Self::new(self.q + 1, self.r),
            Self::new(self.q, self.r + 1),
            Self::new(self.q - 1, self.r + 1),
            Self::new(self.q - 1, self.r),
            Self::new(self.q, self.r - 1),
            Self::new(self.q + 1, self.r - 1),
        ]
    }

    pub fn successors(&self) -> Vec<(Self, u32)> {
        self.neighbours().into_iter().map(|p| (p, 1)).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::map::HexCoord;

    #[test]
    fn distance_to_neighbours() {
        let origin = HexCoord::new(0, 0);
        for neighbour in origin.neighbours() {
            assert_eq!(origin.distance(&neighbour), 1);
        }
    }

    #[test]
    fn long_distance() {
        let origin = HexCoord::new(0, 0);
        let dest = HexCoord::new(4, -2);
        assert_eq!(origin.distance(&dest), 4);
    }
}
