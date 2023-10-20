use super::{component::*, event::*, system::*};
use crate::{
    scene::{SceneSet, SceneState},
    turn::TurnState,
};
use bevy::prelude::*;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SlideEvent>()
            .register_type::<Character>()
            .register_type::<Movement>()
            .register_type::<Enemy>()
            .register_type::<Group>()
            .register_type::<GroupMember>()
            .register_type::<Party>()
            .register_type::<Slide>()
            .add_systems(
                OnEnter(TurnState::Player),
                reset_movement_points.run_if(in_state(SceneState::Active)),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (fluff_party, fluff_enemy).in_set(SceneSet::Populate),
            )
            .add_systems(
                Update,
                (
                    derive_party_movement,
                    despawn_empty_party,
                    slide,
                    update_enemy_visibility,
                ),
            );
    }
}
