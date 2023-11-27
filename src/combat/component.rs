use crate::map::HexCoord;
use bevy::prelude::*;
use smallvec::SmallVec;
use std::ops::Range;

#[derive(Component)]
pub struct Combat {
    pub(super) position: HexCoord,
    pub(super) initiative_order: SmallVec<[Entity; 8]>,
    pub(super) initiative: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health(pub u16, pub u16);

impl Health {
    pub fn heal(&mut self, amount: u16) -> u16 {
        let healed = (self.1 - self.0).min(amount);
        self.0 += healed;
        healed
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Attack {
    pub low: u16,
    pub high: u16,
}

impl Attack {
    pub fn range(&self) -> Range<u16> {
        self.low..self.high
    }
}
