use super::asset::*;
use crate::{assets::CodexAssets, ExplError};
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_codex::Codex;

#[derive(SystemParam)]
pub struct ActorCodex<'w> {
    codex_assets: Res<'w, CodexAssets>,
    actor_codex_assets: Res<'w, Assets<Codex<Actor>>>,
}

impl ActorCodex<'_> {
    pub fn get(&self) -> Result<&Codex<Actor>, ExplError> {
        self.actor_codex_assets
            .get(&self.codex_assets.actor_codex)
            .ok_or(ExplError::MissingCodex)
    }
}
