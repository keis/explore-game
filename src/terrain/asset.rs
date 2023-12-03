use bevy::prelude::*;
pub use expl_codex::{Codex, CodexLoader, CodexSource, Id};
use expl_hexagon::Hexagon;
use serde::Deserialize;

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum TerrainDecoration {
    Water,
    Crystal,
    Tree,
}

#[derive(Clone, Debug, Default, TypePath, Deserialize)]
pub struct Terrain {
    pub symbol: char,
    pub allow_walking: bool,
    pub allow_structure: bool,
    pub height_base: f32,
    pub height_amp: f32,
    pub color_a: Color,
    pub color_b: Color,
    pub color_c: Color,
    #[serde(default)]
    pub decoration: Vec<TerrainDecoration>,
}

impl CodexSource for Terrain {
    const EXTENSION: &'static str = "terrain.toml";
}

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon {
            radius: 1.0,
            subdivisions: 2,
        })),
    });
}
