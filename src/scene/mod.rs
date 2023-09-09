use crate::{
    actor::{
        character::CharacterBundle,
        party::{GroupCommandsExt, PartyBundle, PartyParams},
    },
    assets::AssetState,
    map::{
        spawn_zone, start_map_generation, zone_layer_from_prototype, GenerateMapTask,
        MapCommandsExt, MapPrototype, PresenceLayer, ZoneParams,
    },
    structure::{PortalBundle, PortalParams, SpawnerBundle, SpawnerParams},
};
use bevy::prelude::*;
use futures_lite::future;

mod camera;
mod light;

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
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (
                    start_map_generation,
                    camera::spawn_camera,
                    light::spawn_light,
                ),
            )
            .add_systems(
                Update,
                watch_map_generation_task
                    .run_if(in_state(SceneState::GeneratingMap))
                    .run_if(in_state(AssetState::Loaded)),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    spawn_map.in_set(SceneSet::InitialSetup),
                    apply_deferred.in_set(SceneSet::CommandFlush),
                    (spawn_party, spawn_portal, spawn_spawner).in_set(SceneSet::Populate),
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SceneSet {
    InitialSetup,
    CommandFlush,
    Populate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum SceneState {
    #[default]
    GeneratingMap,
    Active,
}

fn watch_map_generation_task(
    mut commands: Commands,
    mut generate_map_task: Query<(Entity, &mut GenerateMapTask)>,
    mut scene_state: ResMut<NextState<SceneState>>,
) {
    let Ok((entity, mut task)) = generate_map_task.get_single_mut() else { return };
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

pub fn spawn_map(
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
        zone_layer,
        PresenceLayer::new(prototype.tiles.layout),
    ));
}

pub fn spawn_portal(
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
                PortalBundle::new(&mut portal_params, prototype.portal_position),
            ));
        });
}

pub fn spawn_spawner(
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
                .spawn(PartyBundle::new(
                    &mut party_params,
                    prototype.party_position,
                    String::from("Alpha Group"),
                    1,
                ))
                .add_members(&[character1, character2, character3]);
        });
}
