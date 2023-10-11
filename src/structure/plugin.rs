use bevy::prelude::*;

use super::{component::*, system::*};
use crate::{
    assets::AssetState,
    scene::{SceneSet, SceneState},
    turn::TurnState,
};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Camp>()
            .register_type::<Portal>()
            .register_type::<Spawner>()
            .add_systems(Update, (update_camp_view_radius, update_portal_effect))
            .add_systems(
                OnEnter(SceneState::Active),
                (fluff_camp, fluff_portal, fluff_spawner).in_set(SceneSet::Populate),
            )
            .add_systems(OnEnter(TurnState::Player), heal_characters)
            .add_systems(
                OnEnter(TurnState::System),
                (charge_spawner, spawn_enemy)
                    .run_if(in_state(AssetState::Loaded))
                    .chain(),
            );
    }
}
