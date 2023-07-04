use glam::{IVec3, Vec3};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

// sqrt(3)
const SQRT3: f32 = 1.732_050_8;

/// Represents a position on a hexagonal grid with axial coordinates
/// https://www.redblobgames.com/grids/hexagons/#coordinates
#[derive(
    Copy,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Default,
    SerializeDisplay,
    DeserializeFromStr,
)]
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

    pub const fn new_rs(r: i32, s: i32) -> Self {
        HexCoord { q: -s - r, r }
    }

    pub const fn new_qs(q: i32, s: i32) -> Self {
        HexCoord { q, r: -q - s }
    }

    pub fn new_round(q: f32, r: f32) -> Self {
        let qround = q.round();
        let qrem = q - qround;
        let rround = r.round();
        let rrem = r - rround;
        if qrem.abs() >= rrem.abs() {
            Self {
                q: (qround + (qrem + 0.5 * rrem).round()) as i32,
                r: rround as i32,
            }
        } else {
            Self {
                q: qround as i32,
                r: (rround + (rrem + 0.5 * qrem).round()) as i32,
            }
        }
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

impl FromStr for HexCoord {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if !string.starts_with('q') {
            return Err("Coordinate does not start with `q`");
        }
        let rpos = string
            .find('r')
            .ok_or("Coordinate does not have a `r` separator")?;
        let coord = HexCoord::new(
            string[1..rpos]
                .parse()
                .ok()
                .ok_or("Coordinate q-component is not a valid integer")?,
            string[rpos + 1..]
                .parse()
                .ok()
                .ok_or("Coordinate r-component is not a valid integer")?,
        );
        Ok(coord)
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

impl From<HexCoord> for Vec3 {
    fn from(coord: HexCoord) -> Vec3 {
        Self::new(
            ((coord.q as f32) + 0.5 * coord.r as f32) * SQRT3,
            0.0,
            coord.r as f32 * 1.5,
        )
    }
}

impl From<Vec3> for HexCoord {
    fn from(vec: Vec3) -> HexCoord {
        Self::new_round(
            (SQRT3 / 3.0) * vec.x - (1.0 / 3.0) * vec.z,
            (2.0 / 3.0) * vec.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{HexCoord, SQRT3};
    use glam::{IVec3, Vec3};

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

    #[test]
    fn vec3_from_coord() {
        assert_eq!(Vec3::from(HexCoord::ZERO), Vec3::ZERO);
        assert_eq!(Vec3::from(HexCoord::new(1, 0)), Vec3::new(SQRT3, 0.0, 0.0));
        assert_eq!(
            Vec3::from(HexCoord::new(2, 0)),
            Vec3::new(2.0 * SQRT3, 0.0, 0.0)
        );
        assert_eq!(
            Vec3::from(HexCoord::new(0, 1)),
            Vec3::new(0.5 * SQRT3, 0.0, 1.5)
        );
        assert_eq!(
            Vec3::from(HexCoord::new(1, 1)),
            Vec3::new(1.5 * SQRT3, 0.0, 1.5)
        );
    }

    #[test]
    fn convert_2d_point() {
        let coord = HexCoord::new(7, 9);
        let vec: Vec3 = coord.into();
        assert_eq!(coord, HexCoord::from(vec));
        assert_eq!(coord, HexCoord::from(vec + Vec3::new(0.1, 0.0, 0.1)));
        assert_eq!(
            HexCoord::new(8, 9),
            HexCoord::from(vec + Vec3::new(1.0, 0.0, 0.0))
        );
    }

    #[test]
    fn parse_ok() {
        let input = "q10r-2";
        let result: Result<HexCoord, _> = input.parse();
        assert_eq!(result, Ok(HexCoord::new(10, -2)));
    }

    #[test]
    fn parse_fail() {
        let input = "10r-2";
        let result: Result<HexCoord, _> = input.parse();
        assert!(result.is_err());
    }

    #[test]
    fn alternative_constructor() {
        let coord = HexCoord::new(4, 5);
        assert_eq!(coord, HexCoord::new_qs(coord.q, coord.s()));
        assert_eq!(coord, HexCoord::new_rs(coord.r, coord.s()));
    }
}
