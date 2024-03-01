use super::{component::*, event::*, system::*};
use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, HexCoord};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fog>()
            .register_type::<FogRevealer>()
            .register_type::<HexCoord>()
            .register_type::<MapLayout>()
            .register_type::<MapPosition>()
            .register_type::<MapPresence>()
            .register_type::<SquareGridLayout>()
            .register_type::<ViewRadius>()
            .add_systems(
                Update,
                (
                    (update_zone_visibility, log_moves).run_if(on_event::<MapEvent>()),
                    update_terrain_visibility,
                    update_presence_fog,
                ),
            )
            .add_event::<MapEvent>();
    }
}
