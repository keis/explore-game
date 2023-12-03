use super::system::*;
use crate::assets::AssetState;
use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

pub struct FloatingTextPlugin;

impl Plugin for FloatingTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                float_and_fade,
                spawn_floating_text.run_if(on_timer(Duration::from_millis(100))),
            )
                .run_if(in_state(AssetState::Loaded)),
        );
    }
}
