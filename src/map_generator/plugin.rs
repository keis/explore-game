use super::system::*;
use crate::{assets::AssetState, scene::SceneState};
use bevy::prelude::*;

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_map_generation.run_if(in_state(SceneState::Setup)),
                watch_map_generation_task
                    .run_if(in_state(AssetState::Loaded))
                    .run_if(in_state(SceneState::Setup)),
            ),
        );
    }
}
