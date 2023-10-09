use super::{asset::*, command::*, component::*, event::*, hex::*};
use crate::actor::Enemy;
use bevy::prelude::*;

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon {
            radius: 1.0,
            subdivisions: 2,
        })),
    });
}

pub fn log_moves(
    mut map_events: EventReader<MapEvent>,
    presence_query: Query<&MapPresence>,
    presence_layer_query: Query<&PresenceLayer>,
) {
    let Ok(presence_layer) = presence_layer_query.get_single() else { return };
    for event in &mut map_events {
        if let MapEvent::PresenceMoved {
            presence: entity,
            position,
            ..
        } = event
        {
            info!("{:?} moved to {}", entity, position);
            if let Ok(presence) = presence_query.get(*entity) {
                for other in presence_layer
                    .presence(presence.position)
                    .filter(|e| *e != entity)
                {
                    info!("{:?} is here", other);
                }
            }
        }
    }
}

pub fn update_zone_visibility(
    view_query: Query<(&MapPresence, &ViewRadius), Without<Enemy>>,
    mut zone_query: Query<(&MapPosition, &mut Fog)>,
) {
    for (position, mut fog) in zone_query.iter_mut() {
        let visible = view_query
            .iter()
            .any(|(presence, view_radius)| position.0.distance(presence.position) < view_radius.0);

        if visible != fog.visible {
            fog.visible = visible;
            if fog.visible {
                fog.explored = true;
            }
        }
    }
}

pub fn update_terrain_visibility(
    zone_query: Query<(&Children, &Fog), Changed<Fog>>,
    mut terrain_query: Query<(&mut Fog, &mut Visibility), Without<Children>>,
) {
    for (children, parent_fog) in &zone_query {
        let mut terrain_iter = terrain_query.iter_many_mut(children);
        while let Some((mut fog, mut visibility)) = terrain_iter.fetch_next() {
            fog.visible = parent_fog.visible;
            fog.explored = parent_fog.explored;

            *visibility = if fog.explored {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_presence_fog(
    zone_query: Query<(&MapPosition, &Fog), (Changed<Fog>, Without<MapPresence>)>,
    map_query: Query<&PresenceLayer>,
    mut presence_query: Query<(&mut Fog, &mut Visibility), With<MapPresence>>,
) {
    let Ok(presence_layer) = map_query.get_single() else { return };
    for (position, zone_fog) in &zone_query {
        let mut presence_iter = presence_query.iter_many_mut(presence_layer.presence(position.0));
        while let Some((mut fog, mut visibility)) = presence_iter.fetch_next() {
            fog.visible = zone_fog.visible;
            fog.explored = zone_fog.explored;

            if fog.explored {
                *visibility = Visibility::Inherited;
            }
        }
    }
}

pub fn fluff_presence(
    mut commands: Commands,
    map_query: Query<Entity, With<PresenceLayer>>,
    presence_query: Query<(Entity, &MapPresence), Without<GlobalTransform>>,
) {
    let Ok(map_entity) = map_query.get_single() else { return };
    for (entity, presence) in &presence_query {
        commands
            .entity(map_entity)
            .add_presence(entity, presence.position);
    }
}
