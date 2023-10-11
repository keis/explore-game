use crate::{
    actor::{CharacterBundle, GroupCommandsExt, PartyBundle, PartyParams},
    assets::AssetState,
    cleanup,
    input::{action_just_pressed, Action},
    map::{MapCommandsExt, MapLayout, MapPosition, PresenceLayer, ZoneLayer},
    map_generator::{GenerateMapTask, MapPrototype, MapSeed},
    structure::{PortalBundle, PortalParams, SpawnerBundle, SpawnerParams},
    terrain::{CrystalDeposit, Terrain, ZoneBundle, ZoneParams},
    turn::Turn,
};
use bevy::prelude::*;
use expl_hexgrid::{layout::GridLayout, HexCoord};
use expl_wfc::{Seed, SeedType};
use moonshine_save::load::load_from_file;
use std::collections::HashMap;

mod camera;
mod light;
pub mod save;

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
                    camera::spawn_camera,
                    light::spawn_light,
                    load_from_file(save::save_location()),
                    mark_as_loaded.in_set(save::LoadSet::PostLoad),
                ),
            )
            .add_systems(
                Update, // PreUpdate
                save::save_into_file(save::save_location())
                    .run_if(action_just_pressed(Action::Save)),
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
                    cleanup::despawn_all::<(With<save::Save>, Without<Parent>)>,
                    reset_turn_counter,
                    create_map_seed,
                ),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    fluff_loaded_map.in_set(SceneSet::InitialSetup),
                    spawn_generated_map.in_set(SceneSet::InitialSetup),
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
    Reset,
    Active,
}

#[derive(Resource)]
pub struct Loaded;

pub fn has_resource<R: Resource>(resource: Option<Res<R>>) -> bool {
    resource.is_some()
}

fn mark_as_loaded(world: &mut World) {
    world.insert_resource(Loaded);
}

fn move_to_active(mut scene_state: ResMut<NextState<SceneState>>) {
    scene_state.set(SceneState::Active);
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

fn fluff_loaded_map(
    mut commands: Commands,
    map_query: Query<(Entity, &MapLayout)>,
    zone_query: Query<(&MapPosition, Entity), With<Terrain>>,
) {
    let Ok((entity, &MapLayout(layout))) = map_query.get_single() else {
        return;
    };
    let zone_lookup: HashMap<HexCoord, _> = zone_query
        .iter()
        .map(|(&MapPosition(p), e)| (p, e))
        .collect();
    let tiles = layout.iter().map(|coord| zone_lookup[&coord]).collect();
    commands
        .entity(entity)
        .insert((ZoneLayer::new(layout, tiles), PresenceLayer::new(layout)));
}

fn spawn_generated_map(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    map_prototype_query: Query<&MapPrototype>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else {
        return;
    };
    let tiles = prototype
        .tiles
        .iter()
        .map(|(position, zoneproto)| {
            let mut zone = commands.spawn((
                Name::new(format!("Zone {}", position)),
                save::Save,
                ZoneBundle::new(position, zoneproto).with_fluff(&mut zone_params),
            ));

            if zoneproto.crystals {
                zone.insert(CrystalDeposit { amount: 20 });
            }

            zone.id()
        })
        .collect();
    commands.spawn((
        Name::new("Game map"),
        save::Save,
        MapLayout(prototype.tiles.layout),
        ZoneLayer::new(prototype.tiles.layout, tiles),
        PresenceLayer::new(prototype.tiles.layout),
    ));
}

fn spawn_portal(
    mut commands: Commands,
    mut portal_params: PortalParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else {
        return;
    };
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    commands
        .entity(map_entity)
        .with_presence(prototype.portal_position, |location| {
            location.spawn((
                Name::new("Portal"),
                save::Save,
                PortalBundle::new(prototype.portal_position).with_fluff(&mut portal_params),
            ));
        });
}

fn spawn_spawner(
    mut commands: Commands,
    mut spawner_params: SpawnerParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else {
        return;
    };
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    commands
        .entity(map_entity)
        .with_presence(prototype.spawner_position, |location| {
            location.spawn((
                Name::new("EnemySpawner"),
                save::Save,
                SpawnerBundle::new(prototype.spawner_position).with_fluff(&mut spawner_params),
            ));
        });
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: PartyParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(prototype) = map_prototype_query.get_single() else {
        return;
    };
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    let character1 = commands
        .spawn((
            save::Save,
            Name::new("Alice"),
            CharacterBundle::new(String::from("Alice")),
        ))
        .id();
    let character2 = commands
        .spawn((
            save::Save,
            Name::new("Bob"),
            CharacterBundle::new(String::from("Bob")),
        ))
        .id();
    let character3 = commands
        .spawn((
            save::Save,
            Name::new("Carol"),
            CharacterBundle::new(String::from("Carol")),
        ))
        .id();
    commands
        .entity(map_entity)
        .with_presence(prototype.party_position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(prototype.party_position, String::from("Alpha Group"), 1)
                        .with_fluff(&mut party_params),
                ))
                .add_members(&[character1, character2, character3]);
        });
}
