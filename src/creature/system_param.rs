use super::asset::*;
use crate::{assets::CodexAssets, ExplError};
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_codex::Codex;

#[derive(SystemParam)]
pub struct CreatureCodex<'w> {
    codex_assets: Res<'w, CodexAssets>,
    creature_codex_assets: Res<'w, Assets<Codex<Creature>>>,
}

impl<'w> CreatureCodex<'w> {
    pub fn get(&self) -> Result<&Codex<Creature>, ExplError> {
        self.creature_codex_assets
            .get(&self.codex_assets.creature_codex)
            .ok_or(ExplError::MissingCodex)
    }
}
