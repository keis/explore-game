use super::{asset::*, component::*, system::*};
use crate::scene::{SceneSet, SceneState};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Codex<Terrain>>()
            .init_asset::<Codex<Decoration>>()
            .init_asset_loader::<CodexLoader<Terrain>>()
            .init_asset_loader::<CodexLoader<RawDecoration, Decoration>>()
            .register_type::<CrystalDeposit>()
            .register_type::<Height>()
            .register_type::<Height>()
            .register_type::<Option<ZoneDecorationDetail>>()
            .register_type::<OuterVisible>()
            .register_type::<OuterTerrain>()
            .register_type::<TerrainId>()
            .register_type::<Id<Terrain>>()
            .register_type::<Vec<ZoneDecorationDetail>>()
            .register_type::<ZoneDecorationCrystals>()
            .register_type::<ZoneDecorationDetail>()
            .register_type::<ZoneDecorations>()
            .register_type::<ZoneDecorationTree>()
            .register_type::<[f32; 6]>()
            .register_type::<[bool; 6]>()
            .add_systems(Startup, insert_hex_assets)
            .add_systems(
                Update,
                (
                    despawn_empty_crystal_deposit,
                    hide_decorations_behind_camp,
                    show_decorations_behind_camp,
                    update_outer_visible,
                ),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    fluff_zone.map(bevy::utils::warn).in_set(SceneSet::Terrain),
                    decorate_zone
                        .map(bevy::utils::warn)
                        .in_set(SceneSet::Populate),
                ),
            );
    }
}
