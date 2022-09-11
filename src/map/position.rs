use super::HexCoord;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct MapPosition(pub HexCoord);
