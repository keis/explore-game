use bevy::math::IVec3;
use std::ops::Add;

/// Represents a position on a hexagonal grid with axial coordinates
/// https://www.redblobgames.com/grids/hexagons/#coordinates
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    /// Compute the S component of the equivalent cube coordinate
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Get the equivalent cube coordinate
    pub fn qrs(&self) -> IVec3 {
        IVec3::new(self.q, self.r, self.s())
    }

    pub fn distance(&self, other: &Self) -> u32 {
        let diff = HexCoord {
            q: self.q - other.q,
            r: self.r - other.r,
        };
        (diff.q.unsigned_abs() + (diff.q + diff.r).unsigned_abs() + diff.r.unsigned_abs()) / 2
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
}

impl Add for HexCoord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            q: self.q + other.q,
            r: self.r + other.r,
        }
    }
}

/// Construct axial coordinate from a (q,r) tuple
impl From<(i32, i32)> for HexCoord {
    fn from((q, r): (i32, i32)) -> Self {
        HexCoord { q, r }
    }
}

/// Construct axial coordinate from qube coordinate in IVec3
impl TryFrom<IVec3> for HexCoord {
    type Error = &'static str;

    fn try_from(value: IVec3) -> Result<Self, Self::Error> {
        if value.x + value.y + value.z != 0 {
            Err("Components of cube coordinates does not sum to 0")
        } else {
            Ok(HexCoord {
                q: value.x,
                r: value.y,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::map::HexCoord;
    use bevy::math::IVec3;

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

    #[test]
    fn from_tuple() {
        let coord: HexCoord = (4, 5).into();
        assert_eq!(coord, HexCoord { q: 4, r: 5 });
    }

    #[test]
    fn from_ivec3() {
        let coord: Result<HexCoord, _> = IVec3::new(2, -3, 1).try_into();
        assert_eq!(coord, Ok(HexCoord { q: 2, r: -3 }));
    }
}
