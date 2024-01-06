use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct Selection {
    pub is_selected: bool,
}
