use bevy::prelude::*;

use crate::{assets::AssetState, turn::TurnState};

mod camp;
mod portal;
mod spawner;

pub use camp::{Camp, CampBundle, CampParams};
pub use portal::{Portal, PortalBundle, PortalParams};
pub use spawner::{Spawner, SpawnerBundle, SpawnerParams};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (camp::update_camp_view_radius, portal::update_portal_effect),
        )
        .add_systems(
            OnEnter(TurnState::System),
            (spawner::charge_spawner, spawner::spawn_enemy)
                .run_if(in_state(AssetState::Loaded))
                .chain(),
        );
    }
}
