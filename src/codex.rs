//! Generic access to the loaded codexes.
//!
//! Each codex entry type names its handle in [`CodexAssets`] through
//! [`CodexKind`], letting systems and roles look up their codex without a
//! dedicated wrapper type per entry.
use crate::{
    actor::Actor,
    assets::CodexAssets,
    creature::Creature,
    structure::Structure,
    terrain::{Decoration, Terrain},
    ExplError,
};
use bevy::{ecs::system::SystemParam, prelude::*, reflect::TypePath};
use expl_codex::{Codex, CodexSource};
use std::marker::PhantomData;

/// Associates a codex entry type with its handle in [`CodexAssets`].
pub trait CodexKind: CodexSource + TypePath + Sized {
    /// The handle to the codex holding entries of this kind.
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>>;
}

impl CodexKind for Terrain {
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>> {
        &assets.terrain_codex
    }
}

impl CodexKind for Decoration {
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>> {
        &assets.decoration_codex
    }
}

impl CodexKind for Structure {
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>> {
        &assets.structure_codex
    }
}

impl CodexKind for Creature {
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>> {
        &assets.creature_codex
    }
}

impl CodexKind for Actor {
    fn handle(assets: &CodexAssets) -> &Handle<Codex<Self>> {
        &assets.actor_codex
    }
}

/// Looks up the loaded codex of kind `C` from a [`World`].
///
/// Use this outside systems, for example when a [`Role`](crate::role::Role)
/// builds its scene.
pub fn codex<C: CodexKind>(world: &World) -> Result<&Codex<C>, ExplError> {
    let codex_assets = world
        .get_resource::<CodexAssets>()
        .ok_or(ExplError::MissingCodex)?;
    world
        .get_resource::<Assets<Codex<C>>>()
        .and_then(|codexes| codexes.get(C::handle(codex_assets)))
        .ok_or(ExplError::MissingCodex)
}

/// System parameter providing the loaded codex of kind `C`.
#[derive(SystemParam)]
pub struct CodexParam<'w, C: CodexKind> {
    codex_assets: Res<'w, CodexAssets>,
    codexes: Res<'w, Assets<Codex<C>>>,
    _kind: PhantomData<C>,
}

impl<C: CodexKind> CodexParam<'_, C> {
    /// Returns an error when the codex has not been loaded yet.
    pub fn get(&self) -> Result<&Codex<C>, ExplError> {
        self.codexes
            .get(C::handle(&self.codex_assets))
            .ok_or(ExplError::MissingCodex)
    }
}
