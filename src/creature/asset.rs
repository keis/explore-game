use super::component::Attack;
use bevy::prelude::*;
use expl_codex::CodexSource;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, TypePath)]
pub struct Creature {
    pub attack: Attack,
    pub health: u16,
    pub movement: u16,
    pub view_radius: u16,
}

impl CodexSource for Creature {
    const EXTENSION: &'static str = "creature.toml";
}
