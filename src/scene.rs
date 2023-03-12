use crate::{
    character::CharacterBundle,
    enemy::{EnemyBundle, EnemyParams},
    map::{
        spawn_game_map_from_prototype, spawn_zone, GenerateMapTask, HexCoord, MapCommandsExt,
        Terrain, ZoneParams,
    },
    party::{GroupCommandsExt, PartyBundle, PartyParams},
};
use bevy::prelude::*;
use expl_hexgrid::{spiral, GridLayout};
use futures_lite::future;

#[allow(clippy::type_complexity)]
pub fn spawn_scene(
    mut commands: Commands,
    mut params: ParamSet<(PartyParams, ZoneParams, EnemyParams)>,
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

    let map = spawn_game_map_from_prototype(
        &mut commands,
        &prototype,
        |commands, position, zoneproto| spawn_zone(commands, &mut params.p1(), position, zoneproto),
    );

    let groupcoord = spiral(prototype.layout.center())
        .find(|&c| {
            prototype
                .get(c)
                .map_or(false, |proto| proto.terrain != Terrain::Ocean)
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
    commands.entity(map).with_presence(groupcoord, |location| {
        location
            .spawn(PartyBundle::new(
                &mut params.p0(),
                groupcoord,
                String::from("Alpha Group"),
                1,
            ))
            .add_members(&[character1, character2, character3]);
    });

    let enemycoord = spiral(prototype.layout.center() + HexCoord::new(2, 3))
        .find(|&c| {
            prototype
                .get(c)
                .map_or(false, |proto| proto.terrain != Terrain::Ocean)
        })
        .unwrap();
    commands.entity(map).with_presence(enemycoord, |location| {
        location.spawn(EnemyBundle::new(&mut params.p2(), enemycoord));
    });
}
