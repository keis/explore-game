use super::{component::*, event::*, system_param::*};
use crate::{
    action::{ActionPoints, GameAction, GameActionQueue},
    color,
    combat::Combat,
    path::{PathFinder, PathGuided},
    terrain::TerrainId,
};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use bevy_mod_picking::prelude::{Click, Out, Over, Pointer, PointerButton};
use expl_map::{MapPosition, MapPresence};
use std::iter;

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_click_events(
    mut commands: Commands,
    mut click_events: EventReader<Pointer<Click>>,
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
                commands.trigger_targets(ZoneActivated, entity);
                break;
            } else if selection_query.get(entity).is_ok() {
                selection_update.toggle(entity);
                break;
            }
        }
    }
}

pub fn handle_pointer_over_events(
    mut commands: Commands,
    mut events: EventReader<Pointer<Over>>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<Entity, With<Selection>>,
) {
    for event in events.read() {
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                commands.trigger_targets(ZoneOver, entity);
                break;
            } else if selection_query.get(entity).is_ok() {
                commands.trigger_targets(SelectionOver, entity);
                break;
            }
        }
    }
}

pub fn handle_pointer_out_events(
    mut commands: Commands,
    mut events: EventReader<Pointer<Out>>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<Entity, With<Selection>>,
) {
    for event in events.read() {
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                commands.trigger_targets(ZoneOut, entity);
                break;
            } else if selection_query.get(entity).is_ok() {
                commands.trigger_targets(SelectionOut, entity);
                break;
            }
        }
    }
}

pub fn apply_zone_activated_event(
    trigger: Trigger<ZoneActivated>,
    mut presence_query: Query<(
        Entity,
        &MapPresence,
        &ActionPoints,
        &mut PathGuided,
        &Selection,
    )>,
    mut game_action_queue: ResMut<GameActionQueue>,
    zone_query: Query<Entity, With<TerrainId>>,
    combat_query: Query<&Combat>,
    map_position_query: Query<&MapPosition>,
    path_finder: PathFinder,
) {
    if !combat_query.is_empty() {
        return;
    }

    let Ok(target) = zone_query.get(trigger.entity()) else {
        return;
    };
    for (entity, presence, action_points, mut pathguided, _) in presence_query
        .iter_mut()
        .filter(|(_, _, _, _, s)| s.is_selected)
    {
        let Ok(goal) = map_position_query.get(target) else {
            continue;
        };
        let Ok(path_finder) = path_finder.get() else {
            continue;
        };
        let Some(path) = path_finder.find_path(presence.position, goal.0) else {
            continue;
        };
        pathguided.path(path.into_iter().map(|(_, e)| e));
        if action_points.current > 0 {
            if let Some(next) = pathguided.next() {
                game_action_queue.add(GameAction::new_move(entity, *next));
            }
        }
    }
}

pub fn apply_select_event(
    trigger: Trigger<Select>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.entity()) else {
        return;
    };
    selection.is_selected = true;
    for &child in children.iter().flat_map(|c| c.iter()) {
        if let Ok((mut outline_volume, _)) = outline_volume_query.get_mut(child) {
            outline_volume.colour = color::OUTLINE_SELECTED;
        }
    }
}

pub fn apply_deselect_event(
    trigger: Trigger<Deselect>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.entity()) else {
        return;
    };
    selection.is_selected = false;
    for &child in children.iter().flat_map(|c| c.iter()) {
        if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
            *outline_volume = (*default).clone();
        }
    }
}

pub fn apply_selection_over_event(
    trigger: Trigger<SelectionOver>,
    selection_query: Query<(&Selection, &Children)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((_, children)) = selection_query.get(trigger.entity()) else {
        return;
    };
    for &child in children.iter() {
        if let Ok((mut outline_volume, _)) = outline_volume_query.get_mut(child) {
            outline_volume.colour = color::OUTLINE_HOVER;
        }
    }
}

pub fn apply_selection_out_event(
    trigger: Trigger<SelectionOut>,
    selection_query: Query<(&Selection, &Children)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((selection, children)) = selection_query.get(trigger.entity()) else {
        return;
    };
    for &child in children.iter() {
        if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
            if selection.is_selected {
                outline_volume.colour = Color::srgb(0.75, 0.50, 0.50);
            } else {
                *outline_volume = (*default).clone();
            };
        }
    }
}
