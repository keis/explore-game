use super::{
    action::{Action, ActionState},
    component::*,
    event::*,
};
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
    prelude::{Click, GlobalHighlight, Highlight, PickingInteraction, Pointer, PointerButton},
};

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

pub fn send_selection_events(
    action_state: Res<ActionState<Action>>,
    interaction_query: Query<
        (Entity, &Selection, &PickingInteraction),
        Changed<PickingInteraction>,
    >,
    selection_query: Query<(Entity, &Selection)>,
    mut select_events: EventWriter<Select>,
    mut deselect_events: EventWriter<Deselect>,
) {
    for (entity, selection, _) in interaction_query
        .iter()
        .filter(|(_, _, interaction)| matches!(interaction, PickingInteraction::Pressed))
    {
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
    }
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
