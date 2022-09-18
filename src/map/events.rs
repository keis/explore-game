use super::HexCoord;
use bevy::prelude::*;

pub enum MapEvent {
    PresenceAdded {
        map: Entity,
        presence: Entity,
        position: HexCoord,
    },
    PresenceMoved {
        map: Entity,
        presence: Entity,
        position: HexCoord,
    },
}
