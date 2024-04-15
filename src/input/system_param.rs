use super::{action::*, component::*, event::*};
use crate::{actor::Movement, map::MapPresence};
use bevy::{
    ecs::{query::QueryFilter, system::SystemParam},
    prelude::*,
};

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
                if selected.is_some() && movement.current > 0 {
                    return Some(entity);
                }
            }
        }

        for (entity, _, m) in &self.selection_query {
            if selected == Some(entity) {
                break;
            }
            if let Some(movement) = m {
                if movement.current > 0 {
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
    select_events: EventWriter<'w, Select>,
    deselect_events: EventWriter<'w, Deselect>,
}

impl<'w, 's, Filter> SelectionUpdate<'w, 's, Filter>
where
    Filter: QueryFilter + 'static,
{
    pub fn toggle(&mut self, entity: Entity) {
        if let Ok((entity, selection)) = self.selection_query.get(entity) {
            if self.action_state.pressed(&Action::MultiSelect) {
                if selection.is_selected {
                    self.deselect_events.send(Deselect(entity));
                } else {
                    self.select_events.send(Select(entity));
                }
            } else {
                for (other_entity, selection) in self.selection_query.iter() {
                    if other_entity != entity && selection.is_selected {
                        self.deselect_events.send(Deselect(other_entity));
                    }
                }
                if !selection.is_selected {
                    self.select_events.send(Select(entity));
                }
            }
        }
    }
}
