use bevy::prelude::*;

use super::{asset::*, component::*, system::*};
use crate::{
    assets::AssetState,
    scene::{SceneSet, SceneState},
    turn::TurnState,
};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Codex<Structure>>()
            .init_asset_loader::<CodexLoader<RawStructure, Structure>>()
            .register_type::<Camp>()
            .register_type::<Portal>()
            .register_type::<SafeHaven>()
            .register_type::<Spawner>()
            .add_systems(Update, (update_camp_view_radius, update_portal_effect))
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    fluff_camp.map(bevy::utils::warn),
                    fluff_portal.map(bevy::utils::warn),
                    fluff_spawner.map(bevy::utils::warn),
                )
                    .in_set(SceneSet::Populate),
            )
            .add_systems(OnEnter(TurnState::Player), heal_characters)
            .add_systems(
                OnEnter(TurnState::System),
                (charge_spawner, spawn_enemy)
                    .run_if(in_state(AssetState::Loaded))
                    .chain(),
            );
    }
}
