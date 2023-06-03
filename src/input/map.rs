use super::Selection;
use crate::{
    action::{GameAction, GameActionQueue},
    map::{MapPosition, MapPresence, PathGuided, Zone},
};
use bevy::prelude::*;

pub fn handle_zone_interaction(
    zone_query: Query<(&MapPosition, &Interaction), (With<Zone>, Changed<Interaction>)>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    let Ok((zone_position, Interaction::Clicked)) = zone_query.get_single() else { return };
    for (entity, _) in presence_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::MoveTo(entity, zone_position.0));
    }
}
