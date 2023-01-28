use super::HexCoord;
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct MapPosition(pub HexCoord);
