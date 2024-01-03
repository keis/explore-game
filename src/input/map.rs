use super::Selection;
use crate::{
    action::{GameAction, GameActionQueue},
    actor::Movement,
    combat::Combat,
    map::{MapPosition, MapPresence},
    path::{PathFinder, PathGuided},
    terrain::TerrainId,
    ExplError,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, Pointer, PointerButton};

pub fn handle_zone_click_events(
    mut events: EventReader<Pointer<Click>>,
    mut presence_query: Query<(Entity, &MapPresence, &Movement, &mut PathGuided, &Selection)>,
    mut game_action_queue: ResMut<GameActionQueue>,
    zone_query: Query<Entity, With<TerrainId>>,
    combat_query: Query<&Combat>,
    map_position_query: Query<&MapPosition>,
    path_finder: PathFinder,
) -> Result<(), ExplError> {
    if !combat_query.is_empty() {
        events.clear();
        return Ok(());
    }

    for event in events.read() {
        if event.event.button != PointerButton::Primary {
            continue;
        }
        let Ok(target) = zone_query.get(event.target) else {
            continue;
        };
        for (entity, presence, movement, mut pathguided, _) in presence_query
            .iter_mut()
            .filter(|(_, _, _, _, s)| s.is_selected)
        {
            let goal = map_position_query.get(target)?.0;
            let Some(path) = path_finder.get()?.find_path(presence.position, goal) else {
                continue;
            };
            pathguided.path(path.into_iter().map(|(_, e)| e));
            if movement.points > 0 {
                if let Some(next) = pathguided.next() {
                    game_action_queue.add(GameAction::new_move(entity, *next));
                }
            }
        }
    }

    Ok(())
}
