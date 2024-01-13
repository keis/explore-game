use super::component::*;
use crate::{actor::Movement, map::MapPresence};
use bevy::{ecs::system::SystemParam, prelude::*};

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
