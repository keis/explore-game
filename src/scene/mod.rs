use crate::{
    actor::{
        character::CharacterBundle,
        party::{GroupCommandsExt, PartyBundle, PartyParams},
    },
    cleanup,
    map::{spawn_zone, zone_layer_from_prototype, MapCommandsExt, PresenceLayer, ZoneParams},
    map_generator::{GenerateMapTask, MapPrototype, MapSeed},
    structure::{PortalBundle, PortalParams, SpawnerBundle, SpawnerParams},
    turn::Turn,
};
use bevy::prelude::*;
use expl_wfc::{Seed, SeedType};

mod camera;
mod light;
pub mod save;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SceneState>()
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
            .add_systems(Startup, (camera::spawn_camera, light::spawn_light))
            .add_systems(
                OnEnter(SceneState::Setup),
                (
                    cleanup::despawn_all::<(With<save::Save>, Without<Parent>)>,
                    reset_turn_counter,
                    create_map_seed,
                ),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    spawn_map.in_set(SceneSet::InitialSetup),
                    apply_deferred.in_set(SceneSet::CommandFlush),
                    (spawn_party, spawn_portal, spawn_spawner).in_set(SceneSet::Populate),
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
    Active,
}

fn create_map_seed(mut commands: Commands, seed_query: Query<&MapSeed>) {
    if seed_query.is_empty() {
        commands.spawn(MapSeed(Seed::new(SeedType::Square(30, 24))));
    }
}

fn reset_turn_counter(mut turn: ResMut<Turn>) {
    **turn = 1;
}

fn cleanup_map_generation_task(
    mut commands: Commands,
    generate_map_task_query: Query<Entity, With<GenerateMapTask>>,
) {
    for task_entity in &generate_map_task_query {
        commands.entity(task_entity).despawn();
    }
}

fn spawn_map(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    map_prototype_query: Query<&MapPrototype>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else { return };
    let zone_layer =
        zone_layer_from_prototype(&mut commands, prototype, |commands, position, zoneproto| {
            spawn_zone(commands, &mut zone_params, position, zoneproto)
        });
    commands.spawn((
        Name::new("Game map"),
        save::Save,
        zone_layer,
        PresenceLayer::new(prototype.tiles.layout),
    ));
}

fn spawn_portal(
    mut commands: Commands,
    mut portal_params: PortalParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else { return };
    let Ok(map_entity) = map_query.get_single() else { return };
    commands
        .entity(map_entity)
        .with_presence(prototype.portal_position, |location| {
            location.spawn((
                Name::new("Portal"),
                save::Save,
                PortalBundle::new(&mut portal_params, prototype.portal_position),
            ));
        });
}

fn spawn_spawner(
    mut commands: Commands,
    mut spawner_params: SpawnerParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else { return };
    let Ok(map_entity) = map_query.get_single() else { return };
    commands
        .entity(map_entity)
        .with_presence(prototype.spawner_position, |location| {
            location.spawn((
                Name::new("EnemySpawner"),
                save::Save,
                SpawnerBundle::new(&mut spawner_params, prototype.spawner_position),
            ));
        });
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: PartyParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else { return };
    let Ok(map_entity) = map_query.get_single() else { return };
    let character1 = commands
        .spawn(CharacterBundle::new(String::from("Alice")))
        .id();
    let character2 = commands
        .spawn(CharacterBundle::new(String::from("Bob")))
        .id();
    let character3 = commands
        .spawn(CharacterBundle::new(String::from("Carol")))
        .id();
    commands
        .entity(map_entity)
        .with_presence(prototype.party_position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(
                        &mut party_params,
                        prototype.party_position,
                        String::from("Alpha Group"),
                        1,
                    ),
                ))
                .add_members(&[character1, character2, character3]);
        });
}
