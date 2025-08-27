use super::{asset::*, system::*};
use crate::{assets::AssetState, error, scene::SceneState};
use bevy::prelude::*;

pub struct MapGeneratorPlugin;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapTemplate>()
            .init_asset_loader::<TemplateLoader>()
            .add_systems(
                Update,
                (
                    start_map_generation
                        .map(error::warn)
                        .run_if(in_state(SceneState::Reset)),
                    watch_map_generation_task
                        .run_if(in_state(AssetState::Loaded))
                        .run_if(in_state(SceneState::Reset)),
                ),
            );
    }
}
