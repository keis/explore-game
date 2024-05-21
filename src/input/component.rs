use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct Selection {
    pub is_selected: bool,
}

#[derive(Component, Deref, Default)]
pub struct DefaultOutlineVolume(pub OutlineVolume);
