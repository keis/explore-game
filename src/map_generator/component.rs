use crate::{map::Terrain, ExplError};
use bevy::{prelude::*, tasks::Task};
use expl_hexgrid::{layout::SquareGridLayout, Grid, HexCoord};
use expl_wfc::Seed;

#[derive(Default)]
pub struct ZonePrototype {
    pub terrain: Terrain,
    pub random_fill: Vec<(Vec2, f32)>,
    pub crystals: bool,
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
}

#[derive(Component)]
pub struct MapPrototype {
    pub tiles: Grid<SquareGridLayout, ZonePrototype>,
    pub party_position: HexCoord,
    pub portal_position: HexCoord,
    pub spawner_position: HexCoord,
}

#[derive(Component)]
pub struct GenerateMapTask(pub Task<Result<MapPrototype, ExplError>>);

#[derive(Component)]
pub struct MapSeed(pub Seed);