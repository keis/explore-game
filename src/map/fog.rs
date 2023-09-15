use bevy::prelude::*;

#[derive(Component, Reflect, Copy, Clone, Default, Debug)]
#[reflect(Component)]
pub struct Fog {
    pub visible: bool,
    pub explored: bool,
}
