use bevy::{asset::LoadContext, prelude::*};
use expl_codex::{CodexSource, FromWithLoadContext};
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

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RawDecoration {
    color_a: Color,
    color_b: Color,
    color_c: Color,
    mesh: String,
    scale: f32,
}

#[derive(Clone, Debug, Default, TypePath)]
pub struct Decoration {
    pub color_a: Color,
    pub color_b: Color,
    pub color_c: Color,
    pub mesh: Handle<Mesh>,
    pub scale: f32,
}

impl CodexSource for Decoration {
    const EXTENSION: &'static str = "decoration.toml";
}

impl FromWithLoadContext<RawDecoration> for Decoration {
    fn from_with_load_context(raw: RawDecoration, load_context: &mut LoadContext) -> Self {
        Self {
            color_a: raw.color_a,
            color_b: raw.color_b,
            color_c: raw.color_c,
            mesh: load_context.load(raw.mesh),
            scale: raw.scale,
        }
    }
}

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon {
            radius: 1.0,
            subdivisions: 2,
        })),
    });
}
