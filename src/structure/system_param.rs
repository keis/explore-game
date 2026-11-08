use super::asset::*;
use crate::{assets::CodexAssets, ExplError};
use bevy::prelude::*;
use expl_codex::Codex;

/// Looks up the structure codex from a [`World`].
///
/// For callers that are not systems, such as [`Role::attach`](crate::role::Role).
pub fn structure_codex(world: &World) -> Result<&Codex<Structure>, ExplError> {
    let codex_assets = world
        .get_resource::<CodexAssets>()
        .ok_or(ExplError::MissingCodex)?;
    world
        .get_resource::<Assets<Codex<Structure>>>()
        .and_then(|codexes| codexes.get(&codex_assets.structure_codex))
        .ok_or(ExplError::MissingCodex)
}
