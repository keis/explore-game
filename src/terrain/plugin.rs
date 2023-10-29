use super::{asset::*, component::*, system::*};
use crate::scene::{SceneSet, SceneState};
use bevy::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CrystalDeposit>()
            .register_type::<Height>()
            .register_type::<Height>()
            .register_type::<Option<ZoneDecorationDetail>>()
            .register_type::<Outer>()
            .register_type::<Terrain>()
            .register_type::<Vec<ZoneDecorationDetail>>()
            .register_type::<ZoneDecorationCrystals>()
            .register_type::<ZoneDecorationDetail>()
            .register_type::<ZoneDecorations>()
            .register_type::<ZoneDecorationTree>()
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
                    fluff_zone.in_set(SceneSet::Populate),
                    decorate_zone.in_set(SceneSet::Populate),
                ),
            );
    }
}
