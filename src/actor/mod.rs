use bevy::prelude::*;

use crate::turn::Turn;

pub mod character;
pub mod enemy;
pub mod party;
pub mod slide;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<slide::SlideEvent>().add_systems(
            Update,
            (
                character::reset_movement_points.run_if(resource_changed::<Turn>()),
                party::derive_party_movement,
                party::despawn_empty_party,
                enemy::move_enemy.run_if(resource_changed::<Turn>()),
                slide::slide,
            ),
        );
    }
}
