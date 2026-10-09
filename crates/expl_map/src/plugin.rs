use super::{event::*, system::*};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // Non-generic reflected types (Fog, MapPosition, …) auto-register
        // in Bevy 0.17; only generics still need manual registration.
        app.add_systems(
            Update,
            (
                update_zone_visibility,
                log_moves,
                update_terrain_visibility.after(update_zone_visibility),
                update_presence_fog.after(update_zone_visibility),
            )
                .run_if(on_message::<MapEvent>),
        )
        .add_message::<MapEvent>();
    }
}
