use super::{HexCoord, MapPrototype, ZonePrototype};
use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, Grid};
use std::collections::hash_set::HashSet;

#[derive(Component)]
pub struct GameMap {
    tiles: Grid<SquareGridLayout, Entity>,
    presence: Grid<SquareGridLayout, HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl GameMap {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>) -> Self {
        GameMap {
            tiles: Grid::with_data(layout, tiles),
            presence: Grid::new(layout),
            void: HashSet::new(),
        }
    }

    pub fn layout(&self) -> SquareGridLayout {
        self.tiles.layout
    }

    pub fn set(&mut self, position: HexCoord, entity: Entity) {
        self.tiles.set(position, entity)
    }

    pub fn get(&self, position: HexCoord) -> Option<&Entity> {
        self.tiles.get(position)
    }

    pub fn presence(&self, position: HexCoord) -> impl Iterator<Item = &Entity> {
        self.presence
            .get(position)
            .map_or_else(|| self.void.iter(), |presence| presence.iter())
    }

    pub fn add_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.insert(entity);
        }
    }

    pub fn remove_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.remove(&entity);
        }
    }

    pub fn move_presence(&mut self, entity: Entity, origin: HexCoord, destination: HexCoord) {
        if let Some(o) = self.presence.get_mut(origin) {
            o.remove(&entity);
        }
        if let Some(d) = self.presence.get_mut(destination) {
            d.insert(entity);
        }
    }
}

pub fn game_map_from_prototype<F>(
    commands: &mut Commands,
    prototype: &MapPrototype,
    mut spawn_tile: F,
) -> GameMap
where
    F: FnMut(&mut Commands, HexCoord, &ZonePrototype) -> Entity,
{
    GameMap {
        tiles: Grid::with_data(
            prototype.tiles.layout,
            prototype
                .tiles
                .iter()
                .map(|(coord, zoneproto)| spawn_tile(commands, coord, zoneproto)),
        ),
        presence: Grid::new(prototype.tiles.layout),
        void: HashSet::new(),
    }
}
