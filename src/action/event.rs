use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct ActionPointsConsumed {
    pub entity: Entity,
}
