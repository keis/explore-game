use super::save;
use crate::{
    actor::{CharacterBundle, CreatureCodex, CreatureParams, GroupCommandsExt, PartyBundle},
    map::{MapCommandsExt, MapLayout, MapPosition, MapPresence, PresenceLayer, ZoneLayer},
    map_generator::{GenerateMapTask, MapPrototype, MapSeed},
    structure::{PortalBundle, SafeHavenBundle, SpawnerBundle, StructureCodex, StructureParams},
    terrain::{CrystalDeposit, TerrainId, ZoneBundle, ZoneParams},
    turn::Turn,
    ExplError,
};
use bevy::prelude::*;
use expl_codex::Id;
use expl_hexgrid::{layout::GridLayout, HexCoord, Neighbours};
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
    presence_query: Query<(Entity, &MapPresence), Without<GlobalTransform>>,
) -> Result<(), ExplError> {
    let (entity, &MapLayout(layout)) = map_query.get_single()?;
    let zone_lookup: HashMap<HexCoord, _> = zone_query
        .iter()
        .map(|(&MapPosition(p), e)| (p, e))
        .collect();
    let tiles = layout.iter().map(|coord| zone_lookup[&coord]).collect();
    let zone_layer = ZoneLayer::new(layout, tiles);
    let mut presence_layer = PresenceLayer::new(layout);
    for (entity, presence) in &presence_query {
        presence_layer.add_presence(presence.position, entity);
    }
    commands.entity(entity).insert((zone_layer, presence_layer));

    Ok(())
}

pub fn spawn_generated_map(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    map_prototype_query: Query<&MapPrototype>,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let void = Id::from_tag("void");
    let tiles = prototype
        .tiles
        .iter()
        .map(|(position, zoneproto)| {
            let neighbours = Neighbours::from_fn_around(position, |coord| {
                prototype
                    .tiles
                    .get(coord)
                    .map_or(void, |proto| proto.terrain)
            });
            let mut zone = commands.spawn((
                Name::new(format!("Zone {}", position)),
                save::Save,
                ZoneBundle::new(position, zoneproto).with_fluff(&mut zone_params, neighbours),
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
            let (portal_bundle, child_bundle) = PortalBundle::new(prototype.portal_position)
                .with_fluff(&mut structure_params, structure_codex);
            location
                .spawn((Name::new("Portal"), save::Save, portal_bundle))
                .with_children(|parent| {
                    parent.spawn(child_bundle);
                });
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
            let (spawner_bundle, child_bundle) =
                SpawnerBundle::new(prototype.spawner_position, Id::from_tag("slime"))
                    .with_fluff(&mut structure_params, structure_codex);
            location
                .spawn((Name::new("EnemySpawner"), save::Save, spawner_bundle))
                .with_children(|parent| {
                    parent.spawn(child_bundle);
                });
        });

    Ok(())
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: CreatureParams,
    creature_codex: CreatureCodex,
    map_prototype_query: Query<&MapPrototype>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let prototype = map_prototype_query.get_single()?;
    let map_entity = map_query.get_single()?;
    let creature_codex = creature_codex.get()?;

    let character1 = commands
        .spawn((
            save::Save,
            Name::new("Alice"),
            CharacterBundle::new(String::from("Alice"), creature_codex),
        ))
        .id();
    let character2 = commands
        .spawn((
            save::Save,
            Name::new("Bob"),
            CharacterBundle::new(String::from("Bob"), creature_codex),
        ))
        .id();
    let character3 = commands
        .spawn((
            save::Save,
            Name::new("Carol"),
            CharacterBundle::new(String::from("Carol"), creature_codex),
        ))
        .id();
    commands
        .entity(map_entity)
        .with_presence(prototype.party_position, |location| {
            let (party_bundle, child_bundle) =
                PartyBundle::new(prototype.party_position, String::from("Alpha Group"), 1)
                    .with_fluff(&mut party_params, creature_codex);
            location
                .spawn((Name::new("Party"), save::Save, party_bundle))
                .with_children(|parent| {
                    parent.spawn(child_bundle);
                })
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
