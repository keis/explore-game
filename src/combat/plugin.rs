use super::{component::*, event::*, system::*};
use crate::{action::ActionSet, assets::AssetState, map::MapEvent};
use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .register_type::<Attack>()
            .register_type::<Health>()
            .add_systems(
                Update,
                initiate_combat
                    .run_if(on_event::<MapEvent>())
                    .in_set(ActionSet::PostApply),
            )
            .add_systems(
                Update,
                (
                    combat_round.run_if(on_timer(Duration::from_millis(600))),
                    combat_log,
                    spawn_damage_text,
                    make_corpses.after(combat_round),
                    finish_combat.after(make_corpses),
                )
                    .run_if(in_state(AssetState::Loaded)),
            );
    }
}
