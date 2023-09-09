use bevy::prelude::*;

use crate::turn::TurnState;

pub mod character;
pub mod enemy;
pub mod party;
pub mod slide;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<slide::SlideEvent>()
            .add_systems(OnEnter(TurnState::System), enemy::move_enemy)
            .add_systems(OnEnter(TurnState::Player), character::reset_movement_points)
            .add_systems(
                Update,
                (
                    party::derive_party_movement,
                    party::despawn_empty_party,
                    slide::slide,
                ),
            );
    }
}
