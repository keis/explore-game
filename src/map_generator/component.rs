use crate::{terrain::Terrain, ExplError};
use bevy::{prelude::*, tasks::Task};
use expl_codex::Id;
use expl_hexgrid::{layout::SquareGridLayout, Grid, HexCoord};
use expl_wfc::Seed;

#[derive(Default)]
pub struct ZonePrototype {
    pub terrain: Id<Terrain>,
    pub random_fill: Vec<(Vec2, f32)>,
    pub crystals: bool,
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
