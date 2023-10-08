use super::system::*;
use crate::assets::AssetState;
use bevy::prelude::*;

pub struct PathPlugin;

impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_path_display).run_if(in_state(AssetState::Loaded)),
        );
    }
}
