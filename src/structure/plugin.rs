use super::{asset::*, component::*, system::*};
use crate::{
    assets::AssetState,
    error,
    scene::{SceneSet, SceneState},
    turn::TurnState,
};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Codex<Structure>>()
            .init_asset_loader::<CodexLoader<RawStructure, Structure>>()
            .register_type::<Camp>()
            .register_type::<Id<Structure>>()
            .register_type::<Portal>()
            .register_type::<SafeHaven>()
            .register_type::<Spawner>()
            .register_type::<StructureId>()
            .add_systems(Update, (update_camp_view_radius, update_portal_effect))
            .add_systems(
                OnEnter(SceneState::Active),
                fluff_structure.map(error::warn).in_set(SceneSet::Populate),
            )
            .add_systems(OnEnter(TurnState::Player), heal_characters)
            .add_systems(
                OnEnter(TurnState::System),
                (charge_spawner, spawn_enemy.map(error::warn))
                    .run_if(in_state(AssetState::Loaded))
                    .chain(),
            );
    }
}
