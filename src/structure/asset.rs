use bevy::{asset::LoadContext, prelude::*};
use expl_codex::{CodexSource, FromWithLoadContext};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RawStructure {
    color_a: Color,
    color_b: Color,
    color_c: Color,
    mesh: String,
    scale: f32,
    rotation: f32,
}

#[derive(Clone, Debug, Default, TypePath)]
pub struct Structure {
    pub color_a: Color,
    pub color_b: Color,
    pub color_c: Color,
    pub mesh: Handle<Mesh>,
    pub scale: f32,
    pub rotation: f32,
}

impl CodexSource for Structure {
    const EXTENSION: &'static str = "structure.toml";
}

impl FromWithLoadContext<RawStructure> for Structure {
    fn from_with_load_context(raw: RawStructure, load_context: &mut LoadContext) -> Self {
        Self {
            color_a: raw.color_a,
            color_b: raw.color_b,
            color_c: raw.color_c,
            mesh: load_context.load(raw.mesh),
            scale: raw.scale,
            rotation: raw.rotation,
        }
    }
}
