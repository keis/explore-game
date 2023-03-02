use bevy::prelude::*;

#[derive(Component, Copy, Clone, Default, Debug)]
pub struct Fog {
    pub visible: bool,
    pub explored: bool,
}
