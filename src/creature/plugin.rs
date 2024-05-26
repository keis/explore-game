use super::{asset::*, component::*};
use bevy::prelude::*;
use expl_codex::{Codex, CodexLoader, Id};

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Codex<Creature>>()
            .init_asset_loader::<CodexLoader<Creature>>()
            .register_type::<Attack>()
            .register_type::<Corpse>()
            .register_type::<CreatureId>()
            .register_type::<Health>()
            .register_type::<Id<Creature>>()
            .register_type::<Movement>();
    }
}
