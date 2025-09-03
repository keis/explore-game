use crate::{
    assets::AssetState,
    cleanup, error,
    input::{action_just_pressed, Action},
    turn::{TurnSet, TurnState},
};
use bevy::prelude::*;
use bevy_tweening::{component_animator_system, AnimationSystem, TweeningPlugin};
use moonshine_save::{load::load, static_file};

use super::{camera::*, light::*, save::*, score::*, world::*};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .register_type::<Option<Entity>>()
            .add_plugins((
                moonshine_save::save::SavePlugin,
                moonshine_save::load::LoadPlugin,
                TweeningPlugin,
            ))
            .configure_sets(
                OnEnter(SceneState::Active),
                (
                    SceneSet::InitialSetup,
                    SceneSet::InitialFlush,
                    SceneSet::Terrain,
                    SceneSet::TerrainFlush,
                    SceneSet::Populate,
                    SceneSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (
                    spawn_camera,
                    spawn_light,
                    load(static_file(save_location())),
                ),
            )
            .add_systems(
                Update, // PreUpdate
                save_with(filter_with_enabled_components)
                    .into(static_file(save_location()))
                    .run_if(action_just_pressed(Action::Save)),
            )
            .add_systems(
                Update,
                (
                    maybe_mark_as_loaded
                        .run_if(in_state(AssetState::Loaded))
                        .run_if(in_state(SceneState::Setup)),
                    move_to_active
                        .run_if(in_state(AssetState::Loaded))
                        .run_if(in_state(SceneState::Setup))
                        .run_if(has_resource::<Loaded>),
                    (
                        game_over,
                        component_animator_system::<DirectionalLight>
                            .in_set(AnimationSystem::AnimationUpdate),
                    )
                        .run_if(in_state(SceneState::Active)),
                ),
            )
            .add_systems(
                OnEnter(TurnState::Player),
                apply_period_light.map(error::warn).in_set(TurnSet::Effects),
            )
            .add_systems(
                OnEnter(SceneState::Reset),
                (
                    cleanup::despawn_all::<(With<Save>, Without<ChildOf>)>,
                    reset_turn_counter,
                    create_map_seed,
                ),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    fluff_loaded_map
                        .map(error::warn)
                        .in_set(SceneSet::InitialSetup),
                    spawn_generated_map
                        .map(error::warn)
                        .in_set(SceneSet::InitialSetup),
                    ApplyDeferred.in_set(SceneSet::InitialFlush),
                    ApplyDeferred.in_set(SceneSet::TerrainFlush),
                    (
                        spawn_party.map(error::warn),
                        spawn_portal.map(error::warn),
                        spawn_spawner.map(error::warn),
                        spawn_safe_haven.map(error::warn),
                    )
                        .in_set(SceneSet::Populate),
                    cleanup_map_generation_task.in_set(SceneSet::Cleanup),
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SceneSet {
    InitialSetup,
    InitialFlush,
    Terrain,
    TerrainFlush,
    Populate,
    Cleanup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum SceneState {
    #[default]
    Setup,
    Reset,
    Active,
    GameOver,
}

fn has_resource<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some()
}

fn move_to_active(mut scene_state: ResMut<NextState<SceneState>>) {
    scene_state.set(SceneState::Active);
}
