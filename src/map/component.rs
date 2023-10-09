use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, Grid, HexCoord};
use std::collections::hash_set::HashSet;

#[derive(Component, Reflect, Deref, Default)]
#[reflect(Component)]
pub struct MapLayout(pub SquareGridLayout);

#[derive(Component, Reflect, Copy, Clone, Default, Debug)]
#[reflect(Component)]
pub struct Fog {
    pub visible: bool,
    pub explored: bool,
}

#[derive(Component, Reflect, Debug, Default, Deref)]
#[reflect(Component)]
pub struct MapPosition(pub HexCoord);

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct MapPresence {
    pub position: HexCoord,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Offset(pub Vec3);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ViewRadius(pub u32);

#[derive(Component)]
pub struct PresenceLayer {
    presence: Grid<SquareGridLayout, HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl PresenceLayer {
    pub fn new(layout: SquareGridLayout) -> Self {
        PresenceLayer {
            presence: Grid::new(layout),
            void: HashSet::new(),
        }
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

#[derive(Component)]
pub struct ZoneLayer {
    tiles: Grid<SquareGridLayout, Entity>,
}

impl ZoneLayer {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>) -> Self {
        ZoneLayer {
            tiles: Grid::with_data(layout, tiles),
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
}
