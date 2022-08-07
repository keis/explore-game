use crate::hex::HexCoord;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Zone {
    pub position: HexCoord,
}
