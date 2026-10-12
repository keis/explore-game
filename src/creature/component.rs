use super::asset::Creature;
use crate::action::ActionPoints;
use bevy::prelude::*;
use expl_codex::Id;
use serde::Deserialize;
use std::ops::Range;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
#[require(ActionPoints)]
pub struct CreatureId(pub Id<Creature>);

impl CreatureId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Corpse;

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

#[cfg(test)]
mod tests {
    use super::{Attack, CreatureId, Health};
    use crate::action::ActionPoints;
    use bevy::prelude::*;

    #[test]
    fn creature_id_requires_action_points() {
        let mut app = App::new();
        let entity = app.world_mut().spawn(CreatureId::default()).id();
        let entity = app.world().entity(entity);
        assert!(entity.contains::<ActionPoints>());
        // Stats are codex data; zeroed defaults would be lethal.
        assert!(!entity.contains::<Attack>());
        assert!(!entity.contains::<Health>());
    }
}
