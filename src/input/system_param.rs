use super::{action::*, component::*, event::*};
use crate::action::ActionPoints;
use bevy::{
    ecs::{query::QueryFilter, system::SystemParam},
    prelude::*,
};
use expl_map::MapPresence;

#[derive(SystemParam)]
pub struct NextSelectionQuery<'w, 's> {
    selection_query: Query<
        'w,
        's,
        (Entity, &'static Selection, Option<&'static ActionPoints>),
        With<MapPresence>,
    >,
}

impl NextSelectionQuery<'_, '_> {
    pub fn get(&self) -> Option<Entity> {
        let mut selected = None;
        for (entity, selection, action_points) in &self.selection_query {
            if selection.is_selected {
                selected = Some(entity);
            } else if let Some(action_points) = action_points {
                if selected.is_some() && action_points.current > 0 {
                    return Some(entity);
                }
            }
        }

        for (entity, _, action_points) in &self.selection_query {
            if selected == Some(entity) {
                break;
            }
            if let Some(action_points) = action_points {
                if action_points.current > 0 {
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

#[derive(SystemParam)]
pub struct SelectionUpdate<'w, 's, Filter>
where
    Filter: QueryFilter + 'static,
{
    selection_query: Query<'w, 's, (Entity, &'static Selection), Filter>,
    action_state: Res<'w, ActionState<Action>>,
    commands: Commands<'w, 's>,
}

impl<Filter> SelectionUpdate<'_, '_, Filter>
where
    Filter: QueryFilter + 'static,
{
    pub fn toggle(&mut self, entity: Entity) {
        if let Ok((entity, selection)) = self.selection_query.get(entity) {
            if self.action_state.pressed(&Action::MultiSelect) {
                if selection.is_selected {
                    self.commands.trigger_targets(Deselect, entity);
                } else {
                    self.commands.trigger_targets(Select, entity);
                }
            } else {
                for (other_entity, selection) in self.selection_query.iter() {
                    if other_entity != entity && selection.is_selected {
                        self.commands.trigger_targets(Deselect, other_entity);
                    }
                }
                if !selection.is_selected {
                    self.commands.trigger_targets(Select, entity);
                }
            }
        }
    }
}
