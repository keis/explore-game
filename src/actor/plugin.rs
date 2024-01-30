use super::{asset::*, component::*, event::*, system::*};
use crate::{
    scene::{SceneSet, SceneState},
    turn::TurnState,
};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Codex<Creature>>()
            .init_asset_loader::<CodexLoader<RawCreature, Creature>>()
            .add_event::<SlideEvent>()
            .register_type::<Character>()
            .register_type::<CreatureId>()
            .register_type::<Id<Creature>>()
            .register_type::<Corpse>()
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
                (
                    fluff_party.map(bevy::utils::warn),
                    fluff_creature.map(bevy::utils::warn),
                )
                    .in_set(SceneSet::Populate),
            )
            .add_systems(
                Update,
                (
                    derive_party_movement,
                    despawn_empty_party,
                    slide.run_if(in_state(SceneState::Active)),
                    update_enemy_visibility,
                ),
            );
    }
}
