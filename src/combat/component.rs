use crate::map::HexCoord;
use bevy::prelude::*;
use serde::Deserialize;
use smallvec::SmallVec;
use std::ops::Range;

#[derive(Component)]
pub struct Combat {
    pub(super) position: HexCoord,
    pub(super) initiative_order: SmallVec<[Entity; 8]>,
    pub(super) initiative: usize,
}

#[derive(Clone, Debug, Default, Component, Reflect, Deserialize)]
#[reflect(Component)]
pub struct Health {
    pub current: u16,
    pub max: u16,
}

impl Health {
    pub fn heal(&mut self, amount: u16) -> u16 {
        let healed = (self.max - self.current).min(amount);
        self.current += healed;
        healed
    }
}

#[derive(Clone, Debug, Default, Component, Reflect, Deserialize)]
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
