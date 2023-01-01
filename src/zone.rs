use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Terrain {
    Ocean,
    Mountain,
    Forest,
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

#[derive(Component, Copy, Clone, Debug)]
pub struct Zone {
    pub terrain: Terrain,
}
