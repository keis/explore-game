use crate::{
    assets::AssetState,
    cleanup,
    input::{action_just_pressed, Action},
};
use bevy::prelude::*;
use moonshine_save::load::load_from_file;

use super::{camera::*, light::*, save::*, world::*};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SceneState>()
            .register_type::<Option<Entity>>()
            .add_plugins((
                moonshine_save::save::SavePlugin,
                moonshine_save::load::LoadPlugin,
            ))
            .configure_sets(
                OnEnter(SceneState::Active),
                (
                    SceneSet::InitialSetup,
                    SceneSet::CommandFlush,
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
                    load_from_file(save_location()),
                    mark_as_loaded.in_set(LoadSet::PostLoad),
                ),
            )
            .add_systems(
                Update, // PreUpdate
                save_into_file(save_location()).run_if(action_just_pressed(Action::Save)),
            )
            .add_systems(
                Update,
                (move_to_active
                    .run_if(in_state(AssetState::Loaded))
                    .run_if(in_state(SceneState::Setup))
                    .run_if(has_resource::<Loaded>),),
            )
            .add_systems(
                OnEnter(SceneState::Reset),
                (
                    cleanup::despawn_all::<(With<Save>, Without<Parent>)>,
                    reset_turn_counter,
                    create_map_seed,
                ),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    fluff_loaded_map.pipe(warn).in_set(SceneSet::InitialSetup),
                    spawn_generated_map
                        .pipe(warn)
                        .in_set(SceneSet::InitialSetup),
                    apply_deferred.in_set(SceneSet::CommandFlush),
                    (
                        spawn_party.pipe(warn),
                        spawn_portal.pipe(warn),
                        spawn_spawner.pipe(warn),
                        spawn_safe_haven.pipe(warn),
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
    CommandFlush,
    Populate,
    Cleanup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum SceneState {
    #[default]
    Setup,
    Reset,
    Active,
}

fn has_resource<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some()
}

fn move_to_active(mut scene_state: ResMut<NextState<SceneState>>) {
    scene_state.set(SceneState::Active);
}