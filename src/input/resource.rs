use super::event::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SelectedIndex(pub Vec<Entity>);

impl SelectedIndex {
    pub fn on_select(trigger: Trigger<Select>, mut index: ResMut<Self>) {
        index.0.push(trigger.entity())
    }

    pub fn on_deselect(trigger: Trigger<Deselect>, mut index: ResMut<Self>) {
        index.0.retain(|&e| e != trigger.entity());
    }
}
