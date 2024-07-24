use super::HexCoord;
use bevy_ecs::prelude::*;

#[derive(Event)]
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
    PresenceRemoved {
        map: Entity,
        presence: Entity,
        position: HexCoord,
    },
}
