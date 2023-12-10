use super::asset::{Id, Structure};
use bevy::prelude::*;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct StructureId(pub Id<Structure>);

impl StructureId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}
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
