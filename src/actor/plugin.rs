use super::{asset::*, event::*, system::*};
use crate::{
    error,
    scene::{SceneSet, SceneState},
};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SlideEvent>()
            .init_asset::<Codex<Actor>>()
            .init_asset_loader::<CodexLoader<RawActor, Actor>>()
            .register_type::<Id<Actor>>()
            .add_observer(despawn_empty_party.map(error::warn))
            .add_systems(
                OnEnter(SceneState::Active),
                (fluff_party.map(error::warn), fluff_actor.map(error::warn))
                    .in_set(SceneSet::Populate),
            )
            .add_systems(
                Update,
                (
                    slide.run_if(in_state(SceneState::Active)),
                    update_enemy_visibility,
                ),
            );
    }
}
