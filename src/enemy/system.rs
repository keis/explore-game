use super::system_param::*;
use crate::{
    action::{GameAction, GameActionQueue},
    actor::Enemy,
    path::PathFinder,
    ExplError,
};
use bevy::prelude::*;
use expl_map::{HexCoord, MapPresence, PresenceLayer, ViewRadius, ZoneLayer};
use rand::{seq::SliceRandom, thread_rng};

pub fn move_enemy(
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<(&ZoneLayer, &PresenceLayer)>,
    enemy_query: Query<(Entity, &MapPresence, &ViewRadius), With<Enemy>>,
    target: Target,
    path_finder: PathFinder,
) -> Result<(), ExplError> {
    let (zone_layer, presence_layer) = map_query.get_single()?;
    let pf = path_finder.get()?;
    let mut rng = thread_rng();
    for (entity, presence, view_radius) in &enemy_query {
        if let Some(target) = target.closest_in_view(presence.position, view_radius) {
            let Some(path) = pf.find_path(presence.position, target.position) else {
                continue;
            };
            let Some(&(next, next_entity)) = path.get(1) else {
                continue;
            };
            if enemy_query
                .iter_many(presence_layer.presence(next))
                .next()
                .is_some()
            {
                continue;
            }
            queue.add(GameAction::new_move(entity, next_entity));
        } else {
            let mut neighbours = HexCoord::NEIGHBOUR_OFFSETS;
            neighbours.shuffle(&mut rng);
            for offset in neighbours {
                let next = presence.position + offset;
                let next_entity = *zone_layer.get(next).ok_or(ExplError::OutOfBounds)?;
                if pf.is_walkable(next_entity)
                    && enemy_query
                        .iter_many(presence_layer.presence(next))
                        .next()
                        .is_none()
                {
                    queue.add(GameAction::new_move(entity, next_entity));
                    break;
                }
            }
        }
    }
    Ok(())
}
