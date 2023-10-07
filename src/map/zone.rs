use super::HexCoord;
use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, Grid};

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
