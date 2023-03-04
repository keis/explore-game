use super::{GameMap, HexCoord, MapEvent, MapPresence};
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

struct AddMapPresence {
    pub map: Entity,
    pub presence: Entity,
    pub position: HexCoord,
}

struct MoveMapPresence {
    pub map: Entity,
    pub presence: Entity,
    pub position: HexCoord,
}

struct DespawnPresence {
    pub map: Entity,
    pub presence: Entity,
}

pub trait MapCommandsExt {
    fn add_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self;
    fn move_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self;
    fn despawn_presence(&mut self, presence: Entity) -> &mut Self;
}

impl<'w, 's, 'a> MapCommandsExt for EntityCommands<'w, 's, 'a> {
    fn add_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self {
        let map = self.id();
        self.commands().add(AddMapPresence {
            map,
            presence,
            position,
        });
        self
    }

    fn move_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self {
        let map = self.id();
        self.commands().add(MoveMapPresence {
            map,
            presence,
            position,
        });
        self
    }

    fn despawn_presence(&mut self, presence: Entity) -> &mut Self {
        let map = self.id();
        self.commands().add(DespawnPresence { map, presence });
        self
    }
}

impl Command for MoveMapPresence {
    fn write(self, world: &mut World) {
        if let Some(current_position) = world
            .entity(self.presence)
            .get::<MapPresence>()
            .map(|presence| presence.position)
        {
            if let Some(mut map) = world.entity_mut(self.map).get_mut::<GameMap>() {
                map.move_presence(self.presence, current_position, self.position);
            }
        }

        if let Some(mut presence) = world.entity_mut(self.presence).get_mut::<MapPresence>() {
            presence.position = self.position;
        }

        if let Some(mut events) = world.get_resource_mut::<Events<MapEvent>>() {
            events.send(MapEvent::PresenceMoved {
                map: self.map,
                presence: self.presence,
                position: self.position,
            });
        }
    }
}

impl Command for AddMapPresence {
    fn write(self, world: &mut World) {
        // TODO: How to handle the case where presence is already on map? Convert into move?
        if let Some(mut map) = world.entity_mut(self.map).get_mut::<GameMap>() {
            map.add_presence(self.position, self.presence);
        }

        let mut presence_entity = world.entity_mut(self.presence);
        if let Some(mut presence) = presence_entity.get_mut::<MapPresence>() {
            presence.position = self.position;
        } else {
            presence_entity.insert(MapPresence {
                map: self.map,
                position: self.position,
            });
        }

        if let Some(mut events) = world.get_resource_mut::<Events<MapEvent>>() {
            events.send(MapEvent::PresenceAdded {
                map: self.map,
                presence: self.presence,
                position: self.position,
            });
        }
    }
}

impl Command for DespawnPresence {
    fn write(self, world: &mut World) {
        if let Some(position) = world
            .entity(self.presence)
            .get::<MapPresence>()
            .map(|presence| presence.position)
        {
            if let Some(mut map) = world.entity_mut(self.map).get_mut::<GameMap>() {
                map.remove_presence(position, self.presence);
            }
            if let Some(mut events) = world.get_resource_mut::<Events<MapEvent>>() {
                events.send(MapEvent::PresenceRemoved {
                    map: self.map,
                    presence: self.presence,
                    position,
                });
            }
        }

        world.despawn(self.presence);
    }
}
