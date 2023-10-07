use bevy::prelude::*;

use crate::{
    assets::AssetState,
    scene::{SceneSet, SceneState},
    turn::TurnState,
};

mod camp;
mod portal;
mod spawner;

pub use camp::{Camp, CampBundle, CampParams};
pub use portal::{Portal, PortalBundle, PortalParams};
pub use spawner::{Spawner, SpawnerBundle, SpawnerParams};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Camp>()
            .register_type::<Portal>()
            .register_type::<Spawner>()
            .add_systems(
                Update,
                (camp::update_camp_view_radius, portal::update_portal_effect),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (
                    camp::fluff_camp,
                    portal::fluff_portal,
                    spawner::fluff_spawner,
                )
                    .in_set(SceneSet::Populate),
            )
            .add_systems(OnEnter(TurnState::Player), camp::heal_characters)
            .add_systems(
                OnEnter(TurnState::System),
                (spawner::charge_spawner, spawner::spawn_enemy)
                    .run_if(in_state(AssetState::Loaded))
                    .chain(),
            );
    }
}
