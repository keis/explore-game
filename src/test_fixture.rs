use crate::{map::ZoneLayer, terrain::Terrain};
use bevy::prelude::*;
use expl_hexgrid::layout::SquareGridLayout;
use rstest::*;

pub fn spawn_game_map(app: &mut App) -> Entity {
    let tiles = app
        .world
        .spawn_batch(vec![
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Ocean,
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Mountain,
        ])
        .collect();
    app.world
        .spawn(ZoneLayer::new(
            SquareGridLayout {
                width: 3,
                height: 3,
            },
            tiles,
        ))
        .id()
}

#[fixture]
pub fn app() -> App {
    let mut app = App::new();
    spawn_game_map(&mut app);
    app
}
