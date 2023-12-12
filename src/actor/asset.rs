use crate::combat::Attack;
use bevy::{asset::LoadContext, prelude::*};
use expl_codex::{CodexSource, FromWithLoadContext};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RawCreature {
    attack: Attack,
    health: u16,
    movement: u16,
    view_radius: u16,
    color: Color,
    outline_color: Color,
    mesh: String,
    offset: f32,
    scale: f32,
}

#[derive(Clone, Debug, Default, TypePath)]
pub struct Creature {
    pub attack: Attack,
    pub health: u16,
    pub movement: u16,
    pub view_radius: u16,
    pub color: Color,
    pub outline_color: Color,
    pub mesh: Handle<Mesh>,
    pub offset: f32,
    pub scale: f32,
}

impl CodexSource for Creature {
    const EXTENSION: &'static str = "creature.toml";
}

impl FromWithLoadContext<RawCreature> for Creature {
    fn from_with_load_context(raw: RawCreature, load_context: &mut LoadContext) -> Self {
        Self {
            attack: raw.attack,
            health: raw.health,
            movement: raw.movement,
            view_radius: raw.view_radius,
            color: raw.color,
            outline_color: raw.outline_color,
            mesh: load_context.load(raw.mesh),
            offset: raw.offset,
            scale: raw.scale,
        }
    }
}
