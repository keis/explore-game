use super::{action::*, component::*, event::*};
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
    mut select_events: EventWriter<Select>,
    mut deselect_events: EventWriter<Deselect>,
    mut zone_activated_events: EventWriter<ZoneActivated>,
    action_state: Res<ActionState<Action>>,
    parent_query: Query<&Parent>,
    zone_query: Query<Entity, With<MapPosition>>,
    selection_query: Query<(Entity, &Selection)>,
) {
    for event in click_events.read() {
        if event.event.button != PointerButton::Primary {
            continue;
        }
        for entity in iter::once(event.target).chain(parent_query.iter_ancestors(event.target)) {
            if zone_query.get(entity).is_ok() {
                zone_activated_events.send(ZoneActivated(entity));
                break;
            } else if let Ok((entity, selection)) = selection_query.get(entity) {
                if action_state.pressed(Action::MultiSelect) {
                    if selection.is_selected {
                        deselect_events.send(Deselect(entity));
                    } else {
                        select_events.send(Select(entity));
                    }
                } else {
                    for (other_entity, selection) in &selection_query {
                        if entity != other_entity && selection.is_selected {
                            deselect_events.send(Deselect(other_entity));
                        }
                    }
                    if !selection.is_selected {
                        select_events.send(Select(entity));
                    }
                }
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
            if movement.points > 0 {
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
pub fn update_highlight(
    global_defaults: Res<GlobalHighlight<StandardMaterial>>,
    mut interaction_query: Query<
        (
            &mut Handle<StandardMaterial>,
            &PickingInteraction,
            &Selection,
            &InitialHighlight<StandardMaterial>,
            Option<&Highlight<StandardMaterial>>,
        ),
        Or<(Changed<Selection>, Changed<PickingInteraction>)>,
    >,
) {
    for (mut asset, interaction, selection, initial_highlight, highlight) in &mut interaction_query
    {
        if let PickingInteraction::None = interaction {
            *asset = if selection.is_selected {
                global_defaults.selected(&highlight)
            } else {
                initial_highlight.initial.to_owned()
            }
        }
    }
}