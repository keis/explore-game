use super::{component::*, event::*, system_param::*};
use crate::{
    action::{GameAction, GameActionQueue},
    actor::Movement,
    color,
    combat::Combat,
    map::{MapPosition, MapPresence},
    path::{PathFinder, PathGuided},
    terrain::TerrainId,
    ExplError,
};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use bevy_mod_picking::prelude::{Click, Out, Over, Pointer, PointerButton};
use std::iter;

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_click_events(
    mut click_events: EventReader<Pointer<Click>>,
    mut zone_activated_events: EventWriter<ZoneActivated>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<Entity, With<Selection>>,
    mut selection_update: SelectionUpdate<()>,
) {
    for event in click_events.read() {
        if event.event.button != PointerButton::Primary {
            continue;
        }
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                zone_activated_events.send(ZoneActivated(entity));
                break;
            } else if selection_query.get(entity).is_ok() {
                selection_update.toggle(entity);
                break;
            }
        }
    }
}

pub fn handle_pointer_over_events(
    mut events: EventReader<Pointer<Over>>,
    mut zone_over_events: EventWriter<ZoneOver>,
    mut selection_over_events: EventWriter<SelectionOver>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<Entity, With<Selection>>,
) {
    for event in events.read() {
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                zone_over_events.send(ZoneOver(entity));
                break;
            } else if selection_query.get(entity).is_ok() {
                selection_over_events.send(SelectionOver(entity));
                break;
            }
        }
    }
}

pub fn handle_pointer_out_events(
    mut events: EventReader<Pointer<Out>>,
    mut zone_out_events: EventWriter<ZoneOut>,
    mut selection_out_events: EventWriter<SelectionOut>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<Entity, With<Selection>>,
) {
    for event in events.read() {
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                zone_out_events.send(ZoneOut(entity));
                break;
            } else if selection_query.get(entity).is_ok() {
                selection_out_events.send(SelectionOut(entity));
                break;
            }
        }
    }
}

pub fn apply_zone_activated_events(
    mut events: EventReader<ZoneActivated>,
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

    for ZoneActivated(target) in events.read() {
        let Ok(target) = zone_query.get(*target) else {
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
            if movement.current > 0 {
                if let Some(next) = pathguided.next() {
                    game_action_queue.add(GameAction::new_move(entity, *next));
                }
            }
        }
    }

    Ok(())
}

pub fn apply_selection_events(
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut select_events: EventReader<Select>,
    mut deselect_events: EventReader<Deselect>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    for Select(target) in select_events.read() {
        let Ok((mut selection, children)) = selection_query.get_mut(*target) else {
            continue;
        };
        selection.is_selected = true;
        for &child in children.iter().flat_map(|c| c.iter()) {
            if let Ok((mut outline_volume, _)) = outline_volume_query.get_mut(child) {
                outline_volume.colour = color::OUTLINE_SELECTED;
            }
        }
    }

    for Deselect(target) in deselect_events.read() {
        let Ok((mut selection, children)) = selection_query.get_mut(*target) else {
            continue;
        };
        selection.is_selected = false;
        for &child in children.iter().flat_map(|c| c.iter()) {
            if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
                *outline_volume = (*default).clone();
            }
        }
    }
}

pub fn apply_selection_over_out_events(
    selection_query: Query<(&Selection, &Children)>,
    mut selection_over_events: EventReader<SelectionOver>,
    mut selection_out_events: EventReader<SelectionOut>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    for SelectionOver(target) in selection_over_events.read() {
        let Ok((_, children)) = selection_query.get(*target) else {
            continue;
        };
        for &child in children.iter() {
            if let Ok((mut outline_volume, _)) = outline_volume_query.get_mut(child) {
                outline_volume.colour = color::OUTLINE_HOVER;
            }
        }
    }

    for SelectionOut(target) in selection_out_events.read() {
        let Ok((selection, children)) = selection_query.get(*target) else {
            continue;
        };
        for &child in children.iter() {
            if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
                if selection.is_selected {
                    outline_volume.colour = Color::rgb(0.75, 0.50, 0.50);
                } else {
                    *outline_volume = (*default).clone();
                };
            }
        }
    }
}
