use crate::State;
use bevy::{ecs::schedule::ShouldRun, prelude::*};

mod commands;
mod events;
mod fog;
mod gamemap;
mod generator;
mod hex;
mod pathdisplay;
mod pathfinder;
mod pathguided;
mod position;
mod presence;
mod zone;

pub use commands::MapCommandsExt;
pub use events::MapEvent;
pub use expl_hexgrid::HexCoord;
pub use fog::Fog;
pub use gamemap::{spawn_game_map_from_prototype, GameMap};
pub use generator::{start_map_generation, GenerateMapTask, MapPrototype, MapSeed};
pub use hex::{coord_to_vec3, HexAssets};
pub use pathdisplay::PathDisplay;
pub use pathfinder::PathFinder;
pub use pathguided::PathGuided;
pub use position::MapPosition;
pub use presence::{MapPresence, Offset, ViewRadius};
pub use zone::{spawn_zone, Terrain, Zone, ZoneBundle, ZoneParams, ZonePrototype};

#[derive(Resource)]
pub struct Damaged(bool);

fn run_if_damaged(damaged: Res<Damaged>) -> ShouldRun {
    if damaged.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
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
            .add_startup_system(hex::insert_hex_assets)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_damaged)
                    .with_system(presence::update_zone_visibility),
            )
            .add_system_set(
                SystemSet::on_update(State::Running)
                    .with_system(log_moves)
                    .with_system(pathdisplay::update_path_display)
                    .with_system(presence::update_terrain_visibility)
                    .with_system(presence::update_presence_fog)
                    .with_system(presence::update_enemy_visibility)
                    .with_system(zone::despawn_empty_crystal_deposit)
                    .with_system(zone::hide_decorations_behind_camp)
                    .with_system(zone::show_decorations_behind_camp),
            )
            .add_system_to_stage(CoreStage::PostUpdate, damage)
            .add_event::<MapEvent>();
    }
}

fn log_moves(
    mut map_events: EventReader<MapEvent>,
    presence_query: Query<&MapPresence>,
    map_query: Query<&GameMap>,
) {
    for event in map_events.iter() {
        if let MapEvent::PresenceMoved {
            presence: entity,
            position,
            ..
        } = event
        {
            info!("{:?} moved to {}", entity, position);
            if let Ok(presence) = presence_query.get(*entity) {
                if let Ok(map) = map_query.get(presence.map) {
                    for other in map.presence(presence.position).filter(|e| *e != entity) {
                        info!("{:?} is here", other);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GameMap, Terrain, Zone};
    use bevy::prelude::*;
    use expl_hexgrid::layout::SquareGridLayout;
    use rstest::*;

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        let tiles = app
            .world
            .spawn_batch(vec![
                Zone {
                    terrain: Terrain::Forest,
                },
                Zone {
                    terrain: Terrain::Forest,
                },
                Zone {
                    terrain: Terrain::Forest,
                },
                Zone {
                    terrain: Terrain::Ocean,
                },
                Zone {
                    terrain: Terrain::Ocean,
                },
                Zone {
                    terrain: Terrain::Forest,
                },
                Zone {
                    terrain: Terrain::Mountain,
                },
                Zone {
                    terrain: Terrain::Mountain,
                },
                Zone {
                    terrain: Terrain::Mountain,
                },
            ])
            .collect();
        app.world.spawn(GameMap::new(
            SquareGridLayout {
                width: 3,
                height: 3,
            },
            tiles,
        ));
        app
    }
}
