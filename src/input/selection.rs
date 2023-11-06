use crate::{
    actor::Movement,
    input::{Action, ActionState},
    map::MapPresence,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_mod_picking::{
    highlight::InitialHighlight,
    prelude::{GlobalHighlight, Highlight, PickHighlight, Pickable, PickingInteraction},
};

#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct Selection {
    pub is_selected: bool,
}

#[derive(Bundle, Default)]
pub struct SelectionBundle {
    pub pickable: Pickable,
    pub picking_interaction: PickingInteraction,
    pub selection: Selection,
    pub highlight: PickHighlight,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Select(Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Deselect(Entity);

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
    for Select(target) in &mut select_events {
        let Ok(mut selection) = selection_query.get_mut(*target) else {
            continue;
        };
        selection.is_selected = true;
    }

    for Deselect(target) in &mut deselect_events {
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

#[derive(SystemParam)]
pub struct NextSelectionQuery<'w, 's> {
    selection_query:
        Query<'w, 's, (Entity, &'static Selection, Option<&'static Movement>), With<MapPresence>>,
}

impl<'w, 's> NextSelectionQuery<'w, 's> {
    pub fn get(&self) -> Option<Entity> {
        let mut selected = None;
        for (entity, selection, m) in &self.selection_query {
            if selection.is_selected {
                selected = Some(entity);
            } else if let Some(movement) = m {
                if selected.is_some() && movement.points > 0 {
                    return Some(entity);
                }
            }
        }

        for (entity, _, m) in &self.selection_query {
            if selected == Some(entity) {
                break;
            }
            if let Some(movement) = m {
                if movement.points > 0 {
                    if selected == Some(entity) {
                        return None;
                    }
                    return Some(entity);
                }
            }
        }

        None
    }
}
