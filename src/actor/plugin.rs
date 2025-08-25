use super::{asset::*, component::*, event::*, system::*};
use crate::{
    error,
    scene::{SceneSet, SceneState},
};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SlideEvent>()
            .add_event::<MemberAdded>()
            .add_event::<MemberRemoved>()
            .init_asset::<Codex<Actor>>()
            .init_asset_loader::<CodexLoader<RawActor, Actor>>()
            .register_type::<ActorId>()
            .register_type::<Id<Actor>>()
            .register_type::<Character>()
            .register_type::<Enemy>()
            .register_type::<Members>()
            .register_type::<Group>()
            .register_type::<Party>()
            .register_type::<Slide>()
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
