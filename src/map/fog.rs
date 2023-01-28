use bevy::prelude::*;

#[derive(Component, Copy, Clone, Default)]
pub struct Fog {
    pub visible: bool,
    pub explored: bool,
}
