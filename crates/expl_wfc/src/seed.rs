use super::WFCError;
use bincode::Options;
use data_encoding::BASE32_NOPAD;
use expl_hexgrid::layout::{HexagonalGridLayout, SquareGridLayout};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum SeedType {
    Hexagonal(u16),
    Square(u16, u16),
}

impl TryFrom<SeedType> for HexagonalGridLayout {
    type Error = WFCError;

    fn try_from(seed_type: SeedType) -> Result<Self, Self::Error> {
        if let SeedType::Hexagonal(radius) = seed_type {
            Ok(HexagonalGridLayout {
                radius: radius.into(),
            })
        } else {
            Err(WFCError::IncompatibleSeed)
        }
    }
}

impl TryFrom<SeedType> for SquareGridLayout {
    type Error = WFCError;

    fn try_from(seed_type: SeedType) -> Result<Self, Self::Error> {
        if let SeedType::Square(width, height) = seed_type {
            Ok(SquareGridLayout {
                width: width.into(),
                height: height.into(),
            })
        } else {
            Err(WFCError::IncompatibleSeed)
        }
    }
}

impl From<HexagonalGridLayout> for SeedType {
    fn from(layout: HexagonalGridLayout) -> Self {
        SeedType::Hexagonal(layout.radius as u16)
    }
}

impl From<SquareGridLayout> for SeedType {
    fn from(layout: SquareGridLayout) -> Self {
        SeedType::Square(layout.width as u16, layout.height as u16)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Seed {
    pub seed_type: SeedType,
    rng_seed: u64,
}

impl Seed {
    pub fn new(seed_type: SeedType) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            seed_type,
            rng_seed: rng.gen(),
        }
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(bytes) = bincode::config::DefaultOptions::new().serialize(self) {
            write!(f, "{}", BASE32_NOPAD.encode(&bytes))
        } else {
            Err(fmt::Error)
        }
    }
}

impl FromStr for Seed {
    type Err = WFCError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        BASE32_NOPAD
            .decode(string.as_bytes())
            .ok()
            .and_then(|bytes| {
                bincode::config::DefaultOptions::new()
                    .deserialize::<Seed>(&bytes)
                    .ok()
            })
            .ok_or(WFCError::InvalidSeed)
    }
}

impl From<Seed> for rand_xoshiro::Xoshiro256PlusPlus {
    fn from(seed: Seed) -> Self {
        Self::seed_from_u64(seed.rng_seed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Seed, SeedType, WFCError};

    #[test]
    fn display_seed() {
        assert_eq!(
            Seed {
                seed_type: SeedType::Hexagonal(8),
                rng_seed: 1337,
            }
            .to_string(),
            "AAEPWOIF"
        );
    }

    #[test]
    fn parse_seed() -> Result<(), WFCError> {
        let seed: Seed = "AAEPWOIF".parse()?;
        assert_eq!(
            seed,
            Seed {
                seed_type: SeedType::Hexagonal(8),
                rng_seed: 1337,
            }
        );
        Ok(())
    }
}
