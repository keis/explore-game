use super::asset::Creature;
use crate::ExplError;
use bevy::prelude::*;
use expl_codex::Id;
use serde::Deserialize;
use std::ops::Range;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct CreatureId(pub Id<Creature>);

impl CreatureId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Corpse;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Movement {
    pub current: u16,
    pub reset: u16,
}

impl Movement {
    pub fn reset(&mut self) {
        self.current = self.reset;
    }

    pub fn consume(&mut self) -> Result<(), ExplError> {
        if self.current == 0 {
            Err(ExplError::MoveWithoutMovementPoints)
        } else {
            self.current -= 1;
            Ok(())
        }
    }
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
