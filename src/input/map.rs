use super::Selection;
use crate::{
    action::{GameAction, GameActionQueue},
    combat::Combat,
    map::{MapPosition, MapPresence},
    path::PathGuided,
    terrain::TerrainId,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, Pointer, PointerButton};

pub fn handle_zone_click_events(
    mut events: EventReader<Pointer<Click>>,
    zone_query: Query<&MapPosition, With<TerrainId>>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut game_action_queue: ResMut<GameActionQueue>,
    combat_query: Query<&Combat>,
) {
    if !combat_query.is_empty() {
        events.clear();
        return;
    }

    for event in events.read() {
        if event.event.button != PointerButton::Primary {
            continue;
        }
        let Ok(zone_position) = zone_query.get(event.target) else {
            continue;
        };
        for (entity, _) in presence_query.iter().filter(|(_, s)| s.is_selected) {
            game_action_queue.add(GameAction::MoveTo(entity, zone_position.0));
        }
    }
}
