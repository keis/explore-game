use super::HexCoord;
use bevy::prelude::*;

#[derive(Component, Debug, Default, Deref)]
pub struct MapPosition(pub HexCoord);
