use super::HexCoord;
use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default, Deref)]
#[reflect(Component)]
pub struct MapPosition(pub HexCoord);
