use bevy::prelude::*;

use crate::{
    scene::{SceneSet, SceneState},
    turn::TurnState,
};

pub mod character;
pub mod enemy;
pub mod party;
pub mod slide;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<slide::SlideEvent>()
            .register_type::<character::Character>()
            .register_type::<character::Movement>()
            .register_type::<enemy::Enemy>()
            .register_type::<party::Group>()
            .register_type::<party::GroupMember>()
            .register_type::<party::Party>()
            .register_type::<slide::Slide>()
            .add_systems(
                OnEnter(TurnState::System),
                enemy::move_enemy.run_if(in_state(SceneState::Active)),
            )
            .add_systems(
                OnEnter(TurnState::Player),
                character::reset_movement_points.run_if(in_state(SceneState::Active)),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (party::fluff_party, enemy::fluff_enemy).in_set(SceneSet::Populate),
            )
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
