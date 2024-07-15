use super::{asset::MapTemplate, task::generate_map, GenerateMapTask, MapSeed};
use crate::{assets::MainAssets, scene::SceneState, terrain::TerrainCodex, ExplError};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use expl_wfc::Seed;
use futures_lite::future;

pub fn start_map_generation(
    mut commands: Commands,
    seed_query: Query<(Entity, &MapSeed), Without<GenerateMapTask>>,
    main_assets: Res<MainAssets>,
    map_template_assets: Res<Assets<MapTemplate>>,
    terrain_codex: TerrainCodex,
) -> Result<(), ExplError> {
    for (entity, map_seed) in &seed_query {
        let terrain_codex = terrain_codex.get()?.clone();
        let template = map_template_assets
            .get(&main_assets.map_template)
            .ok_or(ExplError::MissingTemplate)?
            .clone();
        let seed: Seed = map_seed.0;
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(async move { generate_map(&terrain_codex, &template, seed) });
        commands.entity(entity).insert(GenerateMapTask(task));
    }
    Ok(())
}

pub fn watch_map_generation_task(
    mut commands: Commands,
    mut generate_map_task: Query<(Entity, &mut GenerateMapTask)>,
    mut scene_state: ResMut<NextState<SceneState>>,
) {
    let Ok((entity, mut task)) = generate_map_task.get_single_mut() else {
        return;
    };
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(prototype)) => {
            commands.entity(entity).insert(prototype);
            scene_state.set(SceneState::Active);
        }
        Some(Err(e)) => {
            error!("something went wrong: {}", e);
        }
        None => (),
    };
}
