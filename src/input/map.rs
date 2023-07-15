use super::Selection;
use crate::{
    action::{GameAction, GameActionQueue},
    map::{MapPosition, MapPresence, PathGuided, Zone},
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, Pointer, PointerButton};

pub fn handle_zone_click_events(
    mut events: EventReader<Pointer<Click>>,
    zone_query: Query<&MapPosition, With<Zone>>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for event in &mut events {
        if event.event.button != PointerButton::Primary {
            continue;
        }
        let Ok(zone_position) = zone_query.get(event.target) else { continue };
        for (entity, _) in presence_query.iter().filter(|(_, s)| s.is_selected) {
            game_action_queue.add(GameAction::MoveTo(entity, zone_position.0));
        }
    }
}
