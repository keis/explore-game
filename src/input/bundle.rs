use super::component::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Pickable, PickingInteraction};

#[derive(Bundle, Default)]
pub struct SelectionBundle {
    pub pickable: Pickable,
    pub picking_interaction: PickingInteraction,
    pub selection: Selection,
}
