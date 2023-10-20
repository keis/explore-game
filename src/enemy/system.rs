use super::system_param::*;
use crate::{
    action::{GameAction, GameActionQueue},
    actor::Enemy,
    map::{HexCoord, MapPresence, PresenceLayer, ViewRadius, ZoneLayer},
    path::PathFinder,
    terrain::Terrain,
};
use bevy::prelude::*;
use rand::{seq::SliceRandom, thread_rng};

pub fn move_enemy(
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<(&PresenceLayer, &ZoneLayer)>,
    enemy_query: Query<(Entity, &MapPresence, &ViewRadius), With<Enemy>>,
    terrain_query: Query<&Terrain>,
    target: Target,
    path_finder: PathFinder,
) {
    let Ok((presence_layer, zone_layer)) = map_query.get_single() else {
        return;
    };
    let mut rng = thread_rng();
    for (entity, presence, view_radius) in &enemy_query {
        if let Some(target) = target.closest_in_view(presence.position, view_radius) {
            let Some((path, _length)) = path_finder.find_path(presence.position, target.position)
            else {
                continue;
            };
            let Some(next) = path.get(1) else { continue };
            if enemy_query
                .iter_many(presence_layer.presence(*next))
                .next()
                .is_some()
            {
                continue;
            }
            queue.add(GameAction::Move(entity, *next));
        } else {
            let mut neighbours = HexCoord::NEIGHBOUR_OFFSETS;
            neighbours.shuffle(&mut rng);
            for offset in neighbours {
                let next = presence.position + offset;
                if zone_layer
                    .get(next)
                    .and_then(|&entity| terrain_query.get(entity).ok())
                    .map_or(false, |terrain| terrain.is_walkable())
                    && enemy_query
                        .iter_many(presence_layer.presence(next))
                        .next()
                        .is_none()
                {
                    queue.add(GameAction::Move(entity, next));
                    break;
                }
            }
        }
    }
}