use crate::{
    character::CharacterBundle,
    enemy::{EnemyBundle, EnemyParams},
    map::{
        spawn_game_map_from_prototype, spawn_zone, GameMap, GenerateMapTask, HexCoord,
        MapCommandsExt, Terrain, Zone, ZoneParams,
    },
    party::{GroupCommandsExt, PartyBundle, PartyParams},
};
use bevy::prelude::*;
use expl_hexgrid::{spiral, GridLayout};
use futures_lite::future;

pub fn spawn_map(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    mut generate_map_task: Query<(Entity, &mut GenerateMapTask)>,
) {
    if generate_map_task.is_empty() {
        return;
    }
    let (task_entity, mut task) = generate_map_task.single_mut();
    let prototype = match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(result)) => {
            commands.entity(task_entity).despawn();
            result
        }
        Some(Err(e)) => {
            error!("something went wrong: {}", e);
            commands.entity(task_entity).despawn();
            return;
        }
        None => return,
    };

    spawn_game_map_from_prototype(
        &mut commands,
        &prototype,
        |commands, position, zoneproto| spawn_zone(commands, &mut zone_params, position, zoneproto),
    );
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: PartyParams,
    map_query: Query<(Entity, &GameMap), Added<GameMap>>,
    zone_query: Query<&Zone>,
) {
    let Ok((map_entity, map)) = map_query.get_single() else { return };
    let groupcoord = spiral(map.layout().center())
        .find(|&c| {
            map.get(c)
                .and_then(|&entity| zone_query.get(entity).ok())
                .map_or(false, |zone| zone.terrain != Terrain::Ocean)
        })
        .unwrap();
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
        .with_presence(groupcoord, |location| {
            location
                .spawn(PartyBundle::new(
                    &mut party_params,
                    groupcoord,
                    String::from("Alpha Group"),
                    1,
                ))
                .add_members(&[character1, character2, character3]);
        });
}

pub fn spawn_enemy(
    mut commands: Commands,
    mut enemy_params: EnemyParams,
    map_query: Query<(Entity, &GameMap), Added<GameMap>>,
    zone_query: Query<&Zone>,
) {
    let Ok((map_entity, map)) = map_query.get_single() else { return };
    let enemycoord = spiral(map.layout().center() + HexCoord::new(2, 3))
        .find(|&c| {
            map.get(c)
                .and_then(|&entity| zone_query.get(entity).ok())
                .map_or(false, |zone| zone.terrain != Terrain::Ocean)
        })
        .unwrap();
    commands
        .entity(map_entity)
        .with_presence(enemycoord, |location| {
            location.spawn(EnemyBundle::new(&mut enemy_params, enemycoord));
        });
}
