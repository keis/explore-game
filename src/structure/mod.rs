use bevy::prelude::*;

use crate::{turn::Turn, State};

mod camp;
mod portal;
mod spawner;

pub use camp::{Camp, CampBundle, CampParams};
pub use portal::{Portal, PortalBundle, PortalParams};
pub use spawner::{Spawner, SpawnerBundle, SpawnerParams};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((camp::update_camp_view_radius, portal::update_portal_effect))
            .add_systems(
                (
                    spawner::charge_spawner.run_if(resource_changed::<Turn>()),
                    spawner::spawn_enemy.run_if(resource_changed::<Turn>()),
                )
                    .chain()
                    .in_set(OnUpdate(State::Running)),
            );
    }
}
