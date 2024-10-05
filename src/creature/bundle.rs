use super::{asset::*, component::*};
use crate::action::ActionPoints;
use bevy::prelude::*;
use expl_codex::{Codex, Id};

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub creature_id: CreatureId,
    pub action_points: ActionPoints,
    pub attack: Attack,
    pub health: Health,
}

impl CreatureBundle {
    pub fn new(creature_codex: &Codex<Creature>, creature_id: Id<Creature>) -> Self {
        let creature_data = &creature_codex[&creature_id];
        Self {
            creature_id: CreatureId(creature_id),
            action_points: ActionPoints::new(creature_data.action_points),
            attack: creature_data.attack.clone(),
            health: Health {
                current: creature_data.health,
                max: creature_data.health,
            },
        }
    }
}
