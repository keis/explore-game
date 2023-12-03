use super::component::*;
use bevy::app::Plugin;
use expl_codex::Id;
use std::collections::HashMap;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Id<Item>>()
            .register_type::<HashMap<Id<Item>, u32>>()
            .register_type::<Inventory>();
    }
}
