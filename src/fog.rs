use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Fog {
    pub visible: bool,
    pub explored: bool,
}
