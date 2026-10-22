use super::WFCError;
use bincode::{Decode, Encode};
use data_encoding::BASE32_NOPAD;
use expl_hexgrid::layout::{HexagonalGridLayout, SquareGridLayout};
use rand::{RngExt, SeedableRng};
use std::{fmt, str::FromStr};

#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq)]
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

#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq)]
pub struct Seed {
    pub seed_type: SeedType,
    rng_seed: u64,
}

impl Seed {
    pub fn new(seed_type: SeedType) -> Self {
        let mut rng = rand::rng();
        Self {
            seed_type,
            rng_seed: rng.random(),
        }
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(bytes) = bincode::encode_to_vec(self, bincode::config::standard()) {
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
                bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .map(|(seed, _)| seed)
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

    /// Pins the wire format used by seed strings. Changing the codec must not
    /// silently change these.
    const SEED_VECTORS: &[(Seed, &str)] = &[
        (
            Seed {
                seed_type: SeedType::Hexagonal(8),
                rng_seed: 1337,
            },
            "AAEPWOIF",
        ),
        (
            Seed {
                seed_type: SeedType::Hexagonal(10),
                rng_seed: 12197247577502074490,
            },
            "AAFP26SGQFDAYVCFVE",
        ),
        (
            Seed {
                seed_type: SeedType::Square(120, 120),
                rng_seed: 9627391014342469595,
            },
            "AF4HR7O3U6VZGHS5TOCQ",
        ),
        (
            Seed {
                seed_type: SeedType::Square(20, 10),
                rng_seed: 299744052673790909,
            },
            "AEKAV7N5IMULPIHHFACA",
        ),
    ];

    #[test]
    fn display_seed() {
        for (seed, encoded) in SEED_VECTORS {
            assert_eq!(seed.to_string(), *encoded);
        }
    }

    #[test]
    fn parse_seed() -> Result<(), WFCError> {
        for (seed, encoded) in SEED_VECTORS {
            let parsed: Seed = encoded.parse()?;
            assert_eq!(parsed, *seed);
        }
        Ok(())
    }

    #[test]
    fn parse_invalid_seed() {
        assert!(matches!(
            "not a seed".parse::<Seed>(),
            Err(WFCError::InvalidSeed)
        ));
    }
}
