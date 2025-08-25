pub struct EnemyPlugin;
use super::system::*;
use crate::{error, scene::SceneState, turn::TurnState};
use bevy::prelude::*;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(TurnState::System),
            move_enemy
                .map(error::warn)
                .run_if(in_state(SceneState::Active)),
        );
    }
}
