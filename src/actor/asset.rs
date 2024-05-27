use bevy::{asset::LoadContext, prelude::*};
use expl_codex::{CodexSource, FromWithLoadContext};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RawActor {
    color: Color,
    outline_color: Color,
    mesh: String,
    offset: f32,
    scale: f32,
}

#[derive(Clone, Debug, Default, TypePath)]
pub struct Actor {
    pub color: Color,
    pub outline_color: Color,
    pub mesh: Handle<Mesh>,
    pub offset: f32,
    pub scale: f32,
}

impl CodexSource for Actor {
    const EXTENSION: &'static str = "actor.toml";
}

impl FromWithLoadContext<RawActor> for Actor {
    fn from_with_load_context(raw: RawActor, load_context: &mut LoadContext) -> Self {
        Self {
            color: raw.color,
            outline_color: raw.outline_color,
            mesh: load_context.load(raw.mesh),
            offset: raw.offset,
            scale: raw.scale,
        }
    }
}
