use crate::{
    assets::CodexAssets,
    map::ZoneLayer,
    terrain::{Terrain, TerrainId},
};
use bevy::prelude::*;
use expl_codex::Codex;
use expl_hexgrid::layout::SquareGridLayout;
use rstest::*;

pub fn spawn_game_map(app: &mut App) -> Entity {
    let tiles = app
        .world_mut()
        .spawn_batch(vec![
            TerrainId::from_tag("forest"),
            TerrainId::from_tag("forest"),
            TerrainId::from_tag("forest"),
            TerrainId::from_tag("ocean"),
            TerrainId::from_tag("ocean"),
            TerrainId::from_tag("forest"),
            TerrainId::from_tag("mountain"),
            TerrainId::from_tag("mountain"),
            TerrainId::from_tag("mountain"),
        ])
        .collect();
    app.world_mut()
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
pub fn default_terrain_codex() -> Codex<Terrain> {
    Codex::from_iter(vec![
        (
            "forest",
            Terrain {
                symbol: '%',
                allow_walking: true,
                ..default()
            },
        ),
        (
            "ocean",
            Terrain {
                symbol: '~',
                allow_walking: false,
                ..default()
            },
        ),
        (
            "mountain",
            Terrain {
                symbol: '^',
                allow_walking: true,
                ..default()
            },
        ),
    ])
}

#[fixture]
pub fn app(default_terrain_codex: Codex<Terrain>) -> App {
    let mut app = App::new();
    let mut terrain_codex_assets: Assets<Codex<Terrain>> = Assets::default();
    let terrain_codex = terrain_codex_assets.add(default_terrain_codex);
    let codex_assets = CodexAssets {
        terrain_codex,
        decoration_codex: Handle::default(),
        structure_codex: Handle::default(),
        creature_codex: Handle::default(),
        actor_codex: Handle::default(),
    };
    app.world_mut().insert_resource(terrain_codex_assets);
    app.world_mut().insert_resource(codex_assets);
    spawn_game_map(&mut app);
    app
}
