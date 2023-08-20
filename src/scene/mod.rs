use crate::{
    character::CharacterBundle,
    map::{
        spawn_zone, zone_layer_from_prototype, GenerateMapTask, Height, MapCommandsExt,
        PresenceLayer, Terrain, Zone, ZoneLayer, ZoneParams,
    },
    party::{GroupCommandsExt, PartyBundle, PartyParams},
    structure::{PortalBundle, PortalParams, SpawnerBundle, SpawnerParams},
    State,
};
use bevy::prelude::*;
use expl_hexgrid::{spiral, GridLayout};
use futures_lite::future;
use glam::Vec3Swizzles;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((spawn_map, spawn_party).in_set(OnUpdate(State::Running)));
    }
}

pub fn spawn_map(
    mut commands: Commands,
    mut param_set: ParamSet<(ZoneParams, PortalParams, SpawnerParams)>,
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
    let zone_layer = zone_layer_from_prototype(
        &mut commands,
        &prototype,
        |commands, position, zoneproto| {
            spawn_zone(commands, &mut param_set.p0(), position, zoneproto)
        },
    );
    commands
        .spawn((
            Name::new("Game map"),
            zone_layer,
            PresenceLayer::new(prototype.tiles.layout),
        ))
        .with_presence(prototype.portal_position, |location| {
            let zone_prototype = prototype.tiles.get(prototype.portal_position).unwrap();
            let height = Height {
                height_amp: zone_prototype.height_amp,
                height_base: zone_prototype.height_base,
                outer_amp: zone_prototype.outer_amp,
                outer_base: zone_prototype.outer_base,
            };
            location.spawn((
                Name::new("Portal"),
                PortalBundle::new(
                    &mut param_set.p1(),
                    prototype.portal_position,
                    height.height_at(Vec2::ZERO, Vec3::from(prototype.portal_position).xz()),
                ),
            ));
        })
        .with_presence(prototype.spawner_position, |location| {
            let zone_prototype = prototype.tiles.get(prototype.portal_position).unwrap();
            let height = Height {
                height_amp: zone_prototype.height_amp,
                height_base: zone_prototype.height_base,
                outer_amp: zone_prototype.outer_amp,
                outer_base: zone_prototype.outer_base,
            };
            location.spawn((
                Name::new("EnemySpawner"),
                SpawnerBundle::new(
                    &mut param_set.p2(),
                    prototype.spawner_position,
                    height.height_at(Vec2::ZERO, Vec3::from(prototype.spawner_position).xz()),
                ),
            ));
        });
}

pub fn spawn_party(
    mut commands: Commands,
    mut party_params: PartyParams,
    map_query: Query<(Entity, &ZoneLayer), Added<ZoneLayer>>,
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
