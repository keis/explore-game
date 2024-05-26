use super::{asset::*, component::*};
use bevy::prelude::*;
use expl_codex::{Codex, Id};

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub creature_id: CreatureId,
    pub movement: Movement,
    pub attack: Attack,
    pub health: Health,
}

impl CreatureBundle {
    pub fn new(creature_codex: &Codex<Creature>, creature_id: Id<Creature>) -> Self {
        let creature_data = &creature_codex[&creature_id];
        Self {
            creature_id: CreatureId(creature_id),
            movement: Movement {
                current: creature_data.movement,
                reset: creature_data.movement,
            },
            attack: creature_data.attack.clone(),
            health: Health {
                current: creature_data.health,
                max: creature_data.health,
            },
        }
    }
}
