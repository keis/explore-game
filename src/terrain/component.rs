use crate::ExplError;
use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{cmp::min, ops::Index};

#[derive(Component, Reflect, Copy, Clone, Debug, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
#[reflect(Component)]
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

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CrystalDeposit {
    pub amount: u8,
}

impl CrystalDeposit {
    pub fn take(&mut self) -> u8 {
        let take = min(self.amount, 10);
        self.amount -= take;
        take
    }
}

#[derive(Reflect, Default, Debug)]
pub struct ZoneDecorationDetail(pub Vec2, pub f32);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct ZoneDecorations {
    pub crystal_detail: Option<ZoneDecorationDetail>,
    pub tree_details: Vec<ZoneDecorationDetail>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZoneDecorationCrystals;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZoneDecorationTree;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Water;

#[derive(Default, Reflect, Copy, Clone)]
pub struct Outer([f32; 6]);

impl Outer {
    pub fn new(self_value: f32, neighbour_values: &[f32]) -> Self {
        let mut data = <[f32; 6]>::default();
        for i in 0..=5 {
            let min_value = self_value
                .min(neighbour_values[i])
                .min(neighbour_values[(i + 1) % 6]);
            let max_value = self_value
                .max(neighbour_values[i])
                .max(neighbour_values[(i + 1) % 6]);

            data[i] = if min_value < 0.0 && max_value < 0.0 {
                max_value
            } else if min_value < 0.0 {
                0.0
            } else {
                min_value
            };
        }
        Self(data)
    }
}

impl Index<usize> for Outer {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<[f32; 6]> for Outer {
    fn from(value: [f32; 6]) -> Self {
        Self(value)
    }
}

impl From<Outer> for [f32; 6] {
    fn from(value: Outer) -> Self {
        value.0
    }
}

#[derive(Component, Default, Deref, Reflect, Copy, Clone)]
pub struct OuterVisible(pub [bool; 6]);

#[derive(Component, Reflect, Default, Copy, Clone)]
#[reflect(Component)]
pub struct Height {
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: Outer,
    pub outer_base: Outer,
}

impl Height {
    const HEX_RADIUS_RATIO: f32 = 0.866_025_4;

    pub(super) fn amp_and_base(&self, local_position: Vec2) -> (f32, f32) {
        let dc = local_position.length();
        let da = (local_position.abs() - Vec2::new(Self::HEX_RADIUS_RATIO, 0.5)).length();
        let db = (local_position.abs() - Vec2::new(0.0, 1.0)).length();

        if dc < 0.7 {
            (self.height_amp, self.height_base)
        } else if da < db {
            if local_position.x > 0.0 {
                if local_position.y > 0.0 {
                    (self.outer_amp[0], self.outer_base[0])
                } else {
                    (self.outer_amp[5], self.outer_base[5])
                }
            } else if local_position.y > 0.0 {
                (self.outer_amp[2], self.outer_base[2])
            } else {
                (self.outer_amp[3], self.outer_base[3])
            }
        } else if local_position.y > 0.0 {
            (self.outer_amp[1], self.outer_base[1])
        } else {
            (self.outer_amp[4], self.outer_base[4])
        }
    }

    pub fn height_at(&self, local_position: Vec2, world_position: Vec2) -> f32 {
        let (amp, base) = self.amp_and_base(local_position);
        let noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
        base + noise * amp
    }
}
