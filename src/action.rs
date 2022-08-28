use crate::map::{find_path, MapComponent, MapPresence, PathGuided};
use crate::HexCoord;
use crate::Terrain;
use crate::Zone;
use bevy::prelude::*;

#[derive(Debug)]
pub enum GameAction {
    MoveTo(Entity, HexCoord),
}

pub fn handle_move_to(
    mut events: EventReader<GameAction>,
    mut presence_query: Query<(&mut PathGuided, &MapPresence)>,
    zone_query: Query<&Zone>,
    map_query: Query<&MapComponent>,
) {
    // Use let_chains after rust 1.64
    for event in events.iter() {
        match event {
            GameAction::MoveTo(e, goal) => {
                if let Ok((mut pathguided, presence)) = presence_query.get_mut(*e) {
                    if let Ok(map) = map_query.get(presence.map) {
                        if let Some((path, _length)) =
                            find_path(presence.position, *goal, &|c: &HexCoord| {
                                if let Some(entity) = map.map.get(*c) {
                                    if let Ok(zone) = zone_query.get(entity) {
                                        return zone.terrain != Terrain::Lava;
                                    }
                                }
                                false
                            })
                        {
                            pathguided.path(path);
                        }
                    }
                }
            }
        }
    }
}
