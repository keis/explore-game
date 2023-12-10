use super::save;
use crate::{
    actor::{CharacterBundle, GroupCommandsExt, PartyBundle, PartyParams},
    map::{MapCommandsExt, MapLayout, MapPosition, PresenceLayer, ZoneLayer},
    map_generator::{GenerateMapTask, MapPrototype, MapSeed},
    structure::{PortalBundle, SafeHavenBundle, SpawnerBundle, StructureCodex, StructureParams},
    terrain::{CrystalDeposit, TerrainCodex, TerrainId, ZoneBundle, ZoneParams},
    turn::Turn,
    ExplError,
};
use bevy::prelude::*;
use expl_hexgrid::{layout::GridLayout, HexCoord};
use expl_wfc::{Seed, SeedType};
use std::collections::HashMap;

pub fn create_map_seed(mut commands: Commands, seed_query: Query<&MapSeed>) {
    if seed_query.is_empty() {
        commands.spawn(MapSeed(Seed::new(SeedType::Square(30, 24))));
    }
}

pub fn reset_turn_counter(mut turn: ResMut<Turn>) {
    **turn = 1;
}

pub fn cleanup_map_generation_task(
    mut commands: Commands,
    generate_map_task_query: Query<Entity, With<GenerateMapTask>>,
) {
    for task_entity in &generate_map_task_query {
        commands.entity(task_entity).despawn();
    }
}

pub fn fluff_loaded_map(
    mut commands: Commands,
    map_query: Query<(Entity, &MapLayout)>,
    zone_query: Query<(&MapPosition, Entity), With<TerrainId>>,
) -> Result<(), ExplError> {
    let (entity, &MapLayout(layout)) = map_query.get_single()?;
    let zone_lookup: HashMap<HexCoord, _> = zone_query
        .iter()
        .map(|(&MapPosition(p), e)| (p, e))
        .collect();
    let tiles = layout.iter().map(|coord| zone_lookup[&coord]).collect();
    commands
        .entity(entity)
        .insert((ZoneLayer::new(layout, tiles), PresenceLayer::new(layout)));

    Ok(())
}

pub fn spawn_generated_map(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    map_prototype_query: Query<&MapPrototype>,
    terrain_codex: TerrainCodex,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let terrain_codex = terrain_codex.get()?;
    let tiles = prototype
        .tiles
        .iter()
        .map(|(position, zoneproto)| {
            let mut zone = commands.spawn((
                Name::new(format!("Zone {}", position)),
                save::Save,
                ZoneBundle::new(position, zoneproto).with_fluff(&mut zone_params, terrain_codex),
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

    Ok(())
}

pub fn spawn_portal(
    mut commands: Commands,
    mut structure_params: StructureParams,
    structure_codex: StructureCodex,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let map_entity = map_query.get_single()?;
    let structure_codex = structure_codex.get()?;

    commands
        .entity(map_entity)
        .with_presence(prototype.portal_position, |location| {
            location.spawn((
                Name::new("Portal"),
                save::Save,
                PortalBundle::new(prototype.portal_position)
                    .with_fluff(&mut structure_params, structure_codex),
            ));
        });

    Ok(())
}

pub fn spawn_spawner(
    mut commands: Commands,
    mut structure_params: StructureParams,
    structure_codex: StructureCodex,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let map_entity = map_query.get_single()?;
    let structure_codex = structure_codex.get()?;

    commands
        .entity(map_entity)
        .with_presence(prototype.spawner_position, |location| {
            location.spawn((
                Name::new("EnemySpawner"),
                save::Save,
                SpawnerBundle::new(prototype.spawner_position)
                    .with_fluff(&mut structure_params, structure_codex),
            ));
        });

    Ok(())
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: PartyParams,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let map_entity = map_query.get_single()?;

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

    Ok(())
}

pub fn spawn_safe_haven(
    mut commands: Commands,
    map_prototype_query: Query<&MapPrototype>,
) -> Result<(), ExplError> {
    // Check for existence of map prototype
    let _prototype = map_prototype_query.get_single()?;

    commands.spawn((
        save::Save,
        Name::new("Safe Haven"),
        SafeHavenBundle::default(),
    ));
    Ok(())
}
