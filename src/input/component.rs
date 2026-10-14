use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

#[derive(Component, Reflect, Debug, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct Selection {
    pub is_selected: bool,
}

#[derive(Component, Reflect, Deref, Default)]
#[reflect(Component)]
pub struct DefaultOutlineVolume(pub OutlineVolume);
