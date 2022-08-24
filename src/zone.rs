use crate::map::HexCoord;
use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Terrain {
    Grass,
    Lava,
}

impl Distribution<Terrain> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Terrain {
        match rng.gen_range(0..=1) {
            0 => Terrain::Grass,
            1 => Terrain::Lava,
            _ => Terrain::Lava,
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Zone {
    pub position: HexCoord,
    pub terrain: Terrain,
}
