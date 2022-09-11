use super::hexcoord::HexCoord;
use super::layout::MapLayout;
use bevy::prelude::*;
use std::collections::hash_set::HashSet;

pub struct MapStorage {
    pub layout: MapLayout,
    tiles: Vec<Option<Entity>>,
    presence: Vec<HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl MapStorage {
    pub fn new(layout: MapLayout) -> Self {
        Self {
            layout,
            tiles: vec![None; layout.size()],
            presence: vec![HashSet::new(); layout.size()],
            void: HashSet::new(),
        }
    }

    pub fn set(&mut self, position: HexCoord, entity: Option<Entity>) {
        if let Some(offset) = self.layout.offset(position) {
            self.tiles[offset] = entity;
        }
    }

    pub fn get(&self, position: HexCoord) -> Option<Entity> {
        self.layout
            .offset(position)
            .and_then(|offset| self.tiles[offset])
    }

    pub fn presence(&self, position: HexCoord) -> impl Iterator<Item = &Entity> {
        self.layout
            .offset(position)
            .map_or_else(|| self.void.iter(), |offset| self.presence[offset].iter())
    }

    pub fn add_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(offset) = self.layout.offset(position) {
            self.presence[offset].insert(entity);
        }
    }

    pub fn remove_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(offset) = self.layout.offset(position) {
            self.presence[offset].remove(&entity);
        }
    }

    pub fn move_presence(&mut self, entity: Entity, origin: HexCoord, destination: HexCoord) {
        // TODO: Consider using let_chains
        if let Some((o, d)) = self
            .layout
            .offset(origin)
            .zip(self.layout.offset(destination))
        {
            self.presence[o].remove(&entity);
            self.presence[d].insert(entity);
        }
    }
}
