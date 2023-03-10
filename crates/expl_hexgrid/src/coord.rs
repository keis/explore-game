use glam::IVec3;
use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Represents a position on a hexagonal grid with axial coordinates
/// https://www.redblobgames.com/grids/hexagons/#coordinates
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const ZERO: HexCoord = HexCoord::new(0, 0);
    pub const NEIGHBOUR_OFFSETS: [HexCoord; 6] = [
        HexCoord::new(1, 0),
        HexCoord::new(0, 1),
        HexCoord::new(-1, 1),
        HexCoord::new(-1, 0),
        HexCoord::new(0, -1),
        HexCoord::new(1, -1),
    ];

    pub const fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    /// Compute the S component of the equivalent cube coordinate
    pub const fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Get the equivalent cube coordinate
    pub fn qrs(&self) -> IVec3 {
        IVec3::new(self.q, self.r, self.s())
    }

    pub fn length(&self) -> u32 {
        (self.q.unsigned_abs() + (self.q + self.r).unsigned_abs() + self.r.unsigned_abs()) / 2
    }

    pub fn distance(&self, other: Self) -> u32 {
        (*self - other).length()
    }

    pub fn neighbours(&self) -> impl '_ + Iterator<Item = HexCoord> {
        HexCoord::NEIGHBOUR_OFFSETS
            .iter()
            .map(|offset| *self + *offset)
    }
}

impl fmt::Display for HexCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q{}r{}", self.q, self.r)
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

impl AddAssign for HexCoord {
    fn add_assign(&mut self, other: Self) {
        self.q += other.q;
        self.r += other.r;
    }
}

impl Sub for HexCoord {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            q: self.q - other.q,
            r: self.r - other.r,
        }
    }
}

impl SubAssign for HexCoord {
    fn sub_assign(&mut self, other: Self) {
        self.q -= other.q;
        self.r -= other.r;
    }
}

impl Mul<i32> for HexCoord {
    type Output = Self;

    fn mul(self, scale: i32) -> Self {
        Self {
            q: self.q * scale,
            r: self.r * scale,
        }
    }
}

impl MulAssign<i32> for HexCoord {
    fn mul_assign(&mut self, scale: i32) {
        self.q *= scale;
        self.r *= scale;
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
    use super::HexCoord;
    use glam::IVec3;

    #[test]
    fn distance_to_neighbours() {
        let origin = HexCoord::ZERO;
        for neighbour in origin.neighbours() {
            assert_eq!(origin.distance(neighbour), 1);
        }
    }

    #[test]
    fn long_distance() {
        let origin = HexCoord::ZERO;
        assert_eq!(origin.distance(HexCoord::new(4, -2)), 4);
        assert_eq!(origin.distance(HexCoord::new(4, -4)), 4);
        assert_eq!(origin.distance(HexCoord::new(3, 3)), 6);
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
