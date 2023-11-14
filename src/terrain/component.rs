use crate::ExplError;
use bevy::prelude::*;
use noisy_bevy::simplex_noise_2d;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    cmp::min,
    ops::{Index, IndexMut},
};

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
    pub fn new(neighbour_values: &[f32]) -> Self {
        let mut data = <[f32; 6]>::default();
        data[..6].copy_from_slice(&neighbour_values[..6]);
        Self(data)
    }

    fn corner(&self, self_value: f32, idx: usize) -> f32 {
        let a = self[idx];
        let b = self[(idx + 1) % 6];

        let min_value = self_value.min(a).min(b);
        let max_value = self_value.max(a).max(b);

        if min_value < 0.0 && max_value < 0.0 {
            max_value
        } else if min_value < 0.0 {
            0.0
        } else {
            min_value
        }
    }

    fn edge(&self, self_value: f32, idx: usize) -> f32 {
        let a = self[idx];

        let min_value = self_value.min(a);
        let max_value = self_value.max(a);

        if min_value < 0.0 && max_value < 0.0 {
            max_value
        } else if min_value < 0.0 {
            0.0
        } else {
            min_value
        }
    }
}

impl Index<usize> for Outer {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Outer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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
    #[reflect(skip_serializing)]
    pub outer_amp: Outer,
    #[reflect(skip_serializing)]
    pub outer_base: Outer,
}

impl Height {
    fn corner(&self, idx: usize) -> (f32, f32) {
        (
            self.outer_amp.corner(self.height_amp, idx),
            self.outer_base.corner(self.height_base, idx),
        )
    }

    fn edge(&self, idx: usize) -> (f32, f32) {
        (
            self.outer_amp.edge(self.height_amp, idx),
            self.outer_base.edge(self.height_base, idx),
        )
    }

    pub(super) fn amp_and_base(&self, local_position: Vec2) -> (f32, f32) {
        // TODO: This could could be simplified by using the fact that the hexagon is symmetrical
        // in both X and Y
        if local_position.y >= 0.9 {
            // South corner
            self.corner(1)
        } else if local_position.y <= -0.9 {
            // North Corner
            self.corner(4)
        } else if local_position.x > 0.0 {
            if local_position.y > local_position.x * -0.5 + 0.9 {
                // South-East Corner or South-East Edge
                if local_position.x > 0.8 {
                    self.corner(0)
                } else {
                    self.edge(1)
                }
            } else if local_position.y < local_position.x * 0.5 - 0.9 {
                // North-East Corner or North-East Edge
                if local_position.x > 0.8 {
                    self.corner(5)
                } else {
                    self.edge(5)
                }
            } else if local_position.x > 0.8 {
                // East Edge
                self.edge(0)
            } else {
                // Internal
                (self.height_amp, self.height_base)
            }
        } else if local_position.x < 0.0 {
            if local_position.y > -local_position.x * -0.5 + 0.9 {
                // South-West Corner or South-West Edge
                if local_position.x < -0.8 {
                    self.corner(2)
                } else {
                    self.edge(2)
                }
            } else if local_position.y < -local_position.x * 0.5 - 0.9 {
                // North-West Corner or North-West Edge
                if local_position.x < -0.8 {
                    self.corner(3)
                } else {
                    self.edge(4)
                }
            } else if local_position.x < 0.8 {
                // West Edge
                self.edge(3)
            } else {
                // Internal
                (self.height_amp, self.height_base)
            }
        } else {
            (self.height_amp, self.height_base)
        }
    }

    pub fn height_at(&self, local_position: Vec2, world_position: Vec2) -> f32 {
        let (amp, base) = self.amp_and_base(local_position);
        let noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
        base + noise * amp
    }
}
