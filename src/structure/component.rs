use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Camp {
    pub name: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct SafeHaven;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Portal {
    pub open: bool,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Spawner {
    pub charge: u8,
}
