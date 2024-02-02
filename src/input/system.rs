use super::{component::*, event::*, system_param::*};
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
use bevy_mod_picking::{
    highlight::InitialHighlight,
    prelude::{
        Click, GlobalHighlight, Highlight, Out, Over, PickingInteraction, Pointer, PointerButton,
    },
};
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
    mut selection_query: Query<&mut Selection>,
    mut select_events: EventReader<Select>,
    mut deselect_events: EventReader<Deselect>,
) {
    for Select(target) in select_events.read() {
        let Ok(mut selection) = selection_query.get_mut(*target) else {
            continue;
        };
        selection.is_selected = true;
    }

    for Deselect(target) in deselect_events.read() {
        let Ok(mut selection) = selection_query.get_mut(*target) else {
            continue;
        };
        selection.is_selected = false;
    }
}

#[allow(clippy::type_complexity)]
pub fn update_selection_highlight(
    global_defaults: Res<GlobalHighlight<StandardMaterial>>,
    selection_query: Query<(Entity, &PickingInteraction, &Selection), Changed<Selection>>,
    children_query: Query<&Children>,
    mut highlight_query: Query<(
        &mut Handle<StandardMaterial>,
        &InitialHighlight<StandardMaterial>,
        Option<&Highlight<StandardMaterial>>,
    )>,
) {
    for (entity, interaction, selection) in &selection_query {
        for entity in iter::once(entity).chain(children_query.iter_descendants(entity)) {
            let Ok((mut handle, initial_highlight, highlight)) = highlight_query.get_mut(entity)
            else {
                continue;
            };
            if let PickingInteraction::None = interaction {
                *handle = if selection.is_selected {
                    global_defaults.selected(&highlight)
                } else {
                    initial_highlight.initial.to_owned()
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_interaction_highlight(
    global_defaults: Res<GlobalHighlight<StandardMaterial>>,
    parent_query: Query<&Parent>,
    selection_query: Query<&Selection>,
    mut interaction_query: Query<
        (
            Entity,
            &mut Handle<StandardMaterial>,
            &InitialHighlight<StandardMaterial>,
            Option<&Highlight<StandardMaterial>>,
            &PickingInteraction,
        ),
        Changed<PickingInteraction>,
    >,
) {
    for (entity, mut handle, initial_highlight, highlight, interaction) in &mut interaction_query {
        if let PickingInteraction::None = interaction {
            for entity in iter::once(entity).chain(parent_query.iter_ancestors(entity)) {
                let Ok(selection) = selection_query.get(entity) else {
                    continue;
                };
                *handle = if selection.is_selected {
                    global_defaults.selected(&highlight)
                } else {
                    initial_highlight.initial.to_owned()
                };
                break;
            }
        }
    }
}
