use crate::ExplError;
use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub enum Terrain {
    #[default]
    Ocean,
    Mountain,
    Forest,
}

impl Terrain {
    pub fn is_walkable(&self) -> bool {
        *self != Terrain::Ocean
    }
}

impl Distribution<Terrain> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Terrain {
        match rng.gen_range(0..=2) {
            0 => Terrain::Ocean,
            1 => Terrain::Mountain,
            2 => Terrain::Forest,
            _ => Terrain::Ocean,
        }
    }
}

impl From<Terrain> for char {
    fn from(terrain: Terrain) -> Self {
        match terrain {
            Terrain::Forest => '%',
            Terrain::Mountain => '^',
            Terrain::Ocean => '~',
        }
    }
}

impl TryFrom<char> for Terrain {
    type Error = ExplError;

    fn try_from(c: char) -> Result<Terrain, Self::Error> {
        match c {
            '%' => Ok(Terrain::Forest),
            '^' => Ok(Terrain::Mountain),
            '~' => Ok(Terrain::Ocean),
            _ => Err(ExplError::UnknownTerrainCharacter),
        }
    }
}

#[derive(Reflect, Default)]
pub struct ZoneDecorationDetail(pub Vec2, pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZoneDecorations {
    pub crystal_detail: Option<ZoneDecorationDetail>,
    pub tree_details: Vec<ZoneDecorationDetail>,
}

#[derive(Component)]
pub struct ZoneDecorationCrystals;

#[derive(Component)]
pub struct ZoneDecorationTree;

#[derive(Component, Default)]
pub struct Water;

#[derive(Default, Copy, Clone)]
pub struct Outer(f32, f32, f32, f32, f32, f32);

impl From<[f32; 6]> for Outer {
    fn from(value: [f32; 6]) -> Self {
        Self(value[0], value[1], value[2], value[3], value[4], value[5])
    }
}

impl From<Outer> for [f32; 6] {
    fn from(value: Outer) -> Self {
        [value.0, value.1, value.2, value.3, value.4, value.5]
    }
}

#[derive(Component, Default, Copy, Clone)]
pub struct Height {
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: Outer,
    pub outer_base: Outer,
}

impl Height {
    const HEX_RADIUS_RATIO: f32 = 0.866_025_4;

    pub fn height_at(&self, local_position: Vec2, world_position: Vec2) -> f32 {
        let dc = local_position.length();
        let da = (local_position.abs() - Vec2::new(Self::HEX_RADIUS_RATIO, 0.5)).length();
        let db = (local_position.abs() - Vec2::new(0.0, 1.0)).length();
        let (amp, base) = if dc < 0.7 {
            (self.height_amp, self.height_base)
        } else if da < db {
            if local_position.x > 0.0 {
                if local_position.y > 0.0 {
                    (self.outer_amp.0, self.outer_base.0)
                } else {
                    (self.outer_amp.5, self.outer_base.5)
                }
            } else if local_position.y > 0.0 {
                (self.outer_amp.2, self.outer_base.2)
            } else {
                (self.outer_amp.3, self.outer_base.3)
            }
        } else if local_position.y > 0.0 {
            (self.outer_amp.1, self.outer_base.1)
        } else {
            (self.outer_amp.4, self.outer_base.4)
        };
        let noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
        base + noise * amp
    }
}
