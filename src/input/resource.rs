use super::{component::Selection, event::*};
use bevy::prelude::*;

/// Handle to the entity holding the global input map, so quill views can
/// look up keybinds without duplicating the map as a resource.
#[derive(Resource)]
pub struct InputMapEntity(pub Entity);

#[derive(Resource, Default)]
pub struct SelectedIndex(pub Vec<Entity>);

impl SelectedIndex {
    pub fn on_select(trigger: On<Select>, mut index: ResMut<Self>) {
        index.0.push(trigger.entity)
    }

    pub fn on_deselect(trigger: On<Deselect>, mut index: ResMut<Self>) {
        index.0.retain(|&e| e != trigger.entity);
    }

    pub fn on_remove(trigger: On<Remove, Selection>, mut index: ResMut<Self>) {
        index.0.retain(|&e| e != trigger.entity);
    }
}

#[derive(Resource, Default)]
pub struct MapHover {
    pub zone: Option<Entity>,
}

impl MapHover {
    pub fn on_zone_over(trigger: On<ZoneOver>, mut map_hover: ResMut<Self>) {
        map_hover.zone = Some(trigger.entity);
    }
}
