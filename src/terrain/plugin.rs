use super::system::*;
use crate::scene::{SceneSet, SceneState};
use bevy::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_empty_crystal_deposit,
                hide_decorations_behind_camp,
                show_decorations_behind_camp,
            ),
        )
        .add_systems(
            OnEnter(SceneState::Active),
            decorate_zone.in_set(SceneSet::Populate),
        );
    }
}
