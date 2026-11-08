use super::asset::*;
use crate::{assets::CodexAssets, ExplError};
use bevy::prelude::*;
use expl_codex::Codex;

/// Looks up the actor codex from a [`World`].
///
/// For callers that are not systems, such as [`Role::attach`](crate::role::Role).
pub fn actor_codex(world: &World) -> Result<&Codex<Actor>, ExplError> {
    let codex_assets = world
        .get_resource::<CodexAssets>()
        .ok_or(ExplError::MissingCodex)?;
    world
        .get_resource::<Assets<Codex<Actor>>>()
        .and_then(|codexes| codexes.get(&codex_assets.actor_codex))
        .ok_or(ExplError::MissingCodex)
}
