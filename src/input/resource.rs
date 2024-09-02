use super::{component::Selection, event::*};
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

    pub fn on_remove(trigger: Trigger<OnRemove, Selection>, mut index: ResMut<Self>) {
        index.0.retain(|&e| e != trigger.entity());
    }
}

#[derive(Resource, Default)]
pub struct MapHover {
    pub zone: Option<Entity>,
}

impl MapHover {
    pub fn on_zone_over(trigger: Trigger<ZoneOver>, mut map_hover: ResMut<Self>) {
        map_hover.zone = Some(trigger.entity());
    }
}
