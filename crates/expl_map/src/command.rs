use super::{HexCoord, MapEvent, MapPresence, PresenceLayer};
use bevy_ecs::{prelude::*, system::EntityCommands, world::Command};
use bevy_hierarchy::despawn_with_children_recursive;
use smallvec::SmallVec;

struct AddMapPresence {
    pub map: Entity,
    pub presence: SmallVec<[Entity; 8]>,
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

pub struct PresenceBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    add_map_presence: AddMapPresence,
}

pub trait MapCommandsExt {
    fn add_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self;
    fn move_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self;
    fn with_presence(
        &mut self,
        position: HexCoord,
        f: impl FnOnce(&mut PresenceBuilder),
    ) -> &mut Self;
    fn despawn_presence(&mut self, presence: Entity) -> &mut Self;
}

impl<'w, 's> PresenceBuilder<'w, 's> {
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let e = self.commands.spawn(bundle);
        self.add_map_presence.presence.push(e.id());
        e
    }
}

impl<'a> MapCommandsExt for EntityCommands<'a> {
    fn add_presence(&mut self, presence: Entity, position: HexCoord) -> &mut Self {
        let map = self.id();
        self.commands().add(AddMapPresence {
            map,
            presence: SmallVec::from_slice(&[presence]),
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

    fn with_presence(
        &mut self,
        position: HexCoord,
        spawn_presence: impl FnOnce(&mut PresenceBuilder),
    ) -> &mut Self {
        let map = self.id();
        let mut builder = PresenceBuilder {
            commands: self.commands(),
            add_map_presence: AddMapPresence {
                map,
                position,
                presence: SmallVec::default(),
            },
        };
        spawn_presence(&mut builder);
        let add_map_presence = builder.add_map_presence;
        self.commands().add(add_map_presence);
        self
    }

    fn despawn_presence(&mut self, presence: Entity) -> &mut Self {
        let map = self.id();
        self.commands().add(DespawnPresence { map, presence });
        self
    }
}

impl Command for MoveMapPresence {
    fn apply(self, world: &mut World) {
        if let Some(current_position) = world
            .entity(self.presence)
            .get::<MapPresence>()
            .map(|presence| presence.position)
        {
            if let Some(mut map) = world.entity_mut(self.map).get_mut::<PresenceLayer>() {
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
    fn apply(self, world: &mut World) {
        for presence in self.presence {
            // TODO: How to handle the case where presence is already on map? Convert into move?
            if let Some(mut map) = world.entity_mut(self.map).get_mut::<PresenceLayer>() {
                map.add_presence(self.position, presence);
            }

            let mut presence_entity = world.entity_mut(presence);
            if let Some(mut presence) = presence_entity.get_mut::<MapPresence>() {
                presence.position = self.position;
            } else {
                presence_entity.insert(MapPresence {
                    position: self.position,
                });
            }

            if let Some(mut events) = world.get_resource_mut::<Events<MapEvent>>() {
                events.send(MapEvent::PresenceAdded {
                    map: self.map,
                    presence,
                    position: self.position,
                });
            }
        }
    }
}

impl Command for DespawnPresence {
    fn apply(self, world: &mut World) {
        if let Some(position) = world
            .entity(self.presence)
            .get::<MapPresence>()
            .map(|presence| presence.position)
        {
            if let Some(mut map) = world.entity_mut(self.map).get_mut::<PresenceLayer>() {
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

        despawn_with_children_recursive(world, self.presence);
    }
}
