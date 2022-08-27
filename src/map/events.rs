use super::HexCoord;
use bevy::prelude::*;

pub struct Entered {
    pub entity: Entity,
    pub coordinate: HexCoord,
}
