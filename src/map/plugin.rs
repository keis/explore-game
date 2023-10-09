use super::{component::*, event::*, resource::*, system::*};
use crate::{
    assets::AssetState,
    scene::{SceneSet, SceneState},
};
use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, HexCoord};
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Damaged(true))
            .register_type::<MapPosition>()
            .register_type::<HexCoord>()
            .register_type::<SquareGridLayout>()
            .register_type::<MapPresence>()
            .register_type::<Offset>()
            .register_type::<ViewRadius>()
            .register_type::<MapLayout>()
            .register_type::<Fog>()
            .add_systems(Startup, insert_hex_assets)
            .add_systems(Update, update_zone_visibility.run_if(run_if_damaged))
            .add_systems(
                Update,
                (log_moves, update_terrain_visibility, update_presence_fog)
                    .run_if(in_state(AssetState::Loaded)),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                fluff_presence.in_set(SceneSet::Populate),
            )
            .add_systems(PostUpdate, damage)
            .add_event::<MapEvent>();
    }
}
