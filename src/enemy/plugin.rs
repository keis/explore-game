pub struct EnemyPlugin;
use super::system::*;
use crate::{scene::SceneState, turn::TurnState};
use bevy::prelude::*;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(TurnState::System),
            move_enemy
                .map(bevy::utils::warn)
                .run_if(in_state(SceneState::Active)),
        );
    }
}
