use super::system::*;
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
        );
    }
}
