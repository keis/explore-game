use crate::{
    assets::AssetState,
    scene::{SceneSet, SceneState},
};
use bevy::prelude::*;

mod commands;
mod events;
mod fog;
mod hex;
mod pathdisplay;
mod pathfinder;
mod pathguided;
mod position;
mod presence;
mod zone;

pub use commands::MapCommandsExt;
pub use events::MapEvent;
pub use expl_hexgrid::{layout::SquareGridLayout, HexCoord};
pub use fog::Fog;
pub use hex::HexAssets;
pub use pathdisplay::PathDisplay;
pub use pathfinder::PathFinder;
pub use pathguided::PathGuided;
pub use position::MapPosition;
pub use presence::{MapPresence, Offset, PresenceLayer, ViewRadius};
pub use zone::ZoneLayer;

#[derive(Resource)]
pub struct Damaged(bool);

#[derive(Component, Reflect, Deref, Default)]
#[reflect(Component)]
pub struct MapLayout(pub SquareGridLayout);

fn run_if_damaged(damaged: Res<Damaged>) -> bool {
    damaged.0
}

fn damage(mut entered_event: EventReader<MapEvent>, mut damaged: ResMut<Damaged>) {
    for _event in entered_event.iter() {
        damaged.0 = true;
    }
}

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
            .add_systems(Startup, hex::insert_hex_assets)
            .add_systems(
                Update,
                presence::update_zone_visibility.run_if(run_if_damaged),
            )
            .add_systems(
                Update,
                (
                    log_moves,
                    pathdisplay::update_path_display,
                    presence::update_terrain_visibility,
                    presence::update_presence_fog,
                    presence::update_enemy_visibility,
                )
                    .run_if(in_state(AssetState::Loaded)),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (presence::fluff_presence.in_set(SceneSet::Populate),),
            )
            .add_systems(PostUpdate, damage)
            .add_event::<MapEvent>();
    }
}

fn log_moves(
    mut map_events: EventReader<MapEvent>,
    presence_query: Query<&MapPresence>,
    presence_layer_query: Query<&PresenceLayer>,
) {
    let Ok(presence_layer) = presence_layer_query.get_single() else { return };
    for event in &mut map_events {
        if let MapEvent::PresenceMoved {
            presence: entity,
            position,
            ..
        } = event
        {
            info!("{:?} moved to {}", entity, position);
            if let Ok(presence) = presence_query.get(*entity) {
                for other in presence_layer
                    .presence(presence.position)
                    .filter(|e| *e != entity)
                {
                    info!("{:?} is here", other);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::ZoneLayer;
    use crate::terrain::Terrain;
    use bevy::prelude::*;
    use expl_hexgrid::layout::SquareGridLayout;
    use rstest::*;

    pub fn spawn_game_map(app: &mut App) -> Entity {
        let tiles = app
            .world
            .spawn_batch(vec![
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
            ])
            .collect();
        app.world
            .spawn(ZoneLayer::new(
                SquareGridLayout {
                    width: 3,
                    height: 3,
                },
                tiles,
            ))
            .id()
    }

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        spawn_game_map(&mut app);
        app
    }
}
