use super::asset::*;
use crate::{assets::CodexAssets, ExplError};
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_codex::Codex;

#[derive(SystemParam)]
pub struct StructureCodex<'w> {
    codex_assets: Res<'w, CodexAssets>,
    structure_codex_assets: Res<'w, Assets<Codex<Structure>>>,
}

impl<'w> StructureCodex<'w> {
    pub fn get(&self) -> Result<&Codex<Structure>, ExplError> {
        self.structure_codex_assets
            .get(&self.codex_assets.structure_codex)
            .ok_or(ExplError::MissingCodex)
    }
}
