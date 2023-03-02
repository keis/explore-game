use crate::State;
use bevy::{
    ecs::{schedule::ShouldRun, system::SystemParam},
    prelude::*,
};
use expl_hexgrid::{layout::SquareGridLayout, Grid};
use pathfinding::prelude::astar;
use std::collections::hash_set::HashSet;

mod commands;
mod events;
mod fog;
mod generator;
mod hex;
mod pathdisplay;
mod pathguided;
mod position;
mod presence;
mod zone;

pub use commands::{AddMapPresence, DespawnPresence, MoveMapPresence};
pub use events::MapEvent;
pub use expl_hexgrid::HexCoord;
pub use fog::Fog;
pub use generator::{start_map_generation, GenerateMapTask, MapPrototype, MapSeed};
pub use hex::{coord_to_vec3, Hexagon};
pub use pathdisplay::PathDisplay;
pub use pathguided::PathGuided;
pub use position::MapPosition;
pub use presence::{MapPresence, Offset, ViewRadius};
pub use zone::{spawn_zone, Terrain, Zone, ZoneBundle, ZonePrototype};

#[derive(Component)]
pub struct GameMap {
    tiles: Grid<SquareGridLayout, Entity>,
    presence: Grid<SquareGridLayout, HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl GameMap {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>) -> Self {
        GameMap {
            tiles: Grid::with_data(layout, tiles),
            presence: Grid::new(layout),
            void: HashSet::new(),
        }
    }

    pub fn set(&mut self, position: HexCoord, entity: Entity) {
        self.tiles.set(position, entity)
    }

    pub fn get(&self, position: HexCoord) -> Option<&Entity> {
        self.tiles.get(position)
    }

    pub fn presence(&self, position: HexCoord) -> impl Iterator<Item = &Entity> {
        self.presence
            .get(position)
            .map_or_else(|| self.void.iter(), |presence| presence.iter())
    }

    pub fn add_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.insert(entity);
        }
    }

    pub fn remove_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.remove(&entity);
        }
    }

    pub fn move_presence(&mut self, entity: Entity, origin: HexCoord, destination: HexCoord) {
        if let Some(o) = self.presence.get_mut(origin) {
            o.remove(&entity);
        }
        if let Some(d) = self.presence.get_mut(destination) {
            d.insert(entity);
        }
    }
}

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

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon { radius: 1.0 })),
    });
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Damaged(true))
            .add_startup_system(insert_hex_assets)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_damaged)
                    .with_system(presence::update_zone_visibility),
            )
            .add_system_set(
                SystemSet::on_update(State::Running)
                    .with_system(pathdisplay::update_path_display)
                    .with_system(presence::update_terrain_visibility)
                    .with_system(presence::update_enemy_visibility)
                    .with_system(zone::despawn_empty_crystal_deposit),
            )
            .add_system_to_stage(CoreStage::PostUpdate, damage)
            .add_event::<MapEvent>();
    }
}

#[derive(SystemParam)]
pub struct PathFinder<'w, 's> {
    map_query: Query<'w, 's, &'static GameMap>,
    zone_query: Query<'w, 's, &'static Zone>,
}

impl<'w, 's> PathFinder<'w, 's> {
    pub fn find_path(&self, start: HexCoord, goal: HexCoord) -> Option<(Vec<HexCoord>, u32)> {
        let map = self.map_query.single();
        astar(
            &start,
            |p| {
                p.neighbours()
                    .filter(&|c: &HexCoord| {
                        map.get(*c)
                            .and_then(|&entity| self.zone_query.get(entity).ok())
                            .map_or(false, |zone| zone.is_walkable())
                    })
                    .map(|p| (p, 1))
                    .collect::<Vec<(HexCoord, u32)>>()
            },
            |p| p.distance(goal),
            |p| *p == goal,
        )
    }
}

pub fn spawn_game_map_from_prototype<F>(
    commands: &mut Commands,
    prototype: &MapPrototype,
    mut spawn_tile: F,
) -> Entity
where
    F: FnMut(&mut Commands, HexCoord, &ZonePrototype) -> Entity,
{
    let gamemap = GameMap {
        tiles: Grid::with_data(
            prototype.layout,
            prototype
                .iter()
                .map(|(coord, zoneproto)| spawn_tile(commands, coord, zoneproto)),
        ),
        presence: Grid::new(prototype.layout),
        void: HashSet::new(),
    };
    commands.spawn(gamemap).id()
}

#[cfg(test)]
mod tests {
    use super::{GameMap, HexCoord, PathFinder, SquareGridLayout, Terrain, Zone};
    use bevy::prelude::*;
    use rstest::*;

    #[fixture]
    fn app() -> App {
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

    #[derive(Component, Debug)]
    struct Goal(HexCoord);

    #[derive(Component, Debug)]
    struct Start(HexCoord);

    #[derive(Component, Debug)]
    struct Path(Vec<HexCoord>, u32);

    fn find_path_system(
        mut commands: Commands,
        path_finder: PathFinder,
        params_query: Query<(Entity, &Start, &Goal)>,
    ) {
        let (entity, start, goal) = params_query.single();
        if let Some(path) = path_finder.find_path(start.0, goal.0) {
            commands.entity(entity).insert(Path(path.0, path.1));
        } else {
            println!("WHAT");
        }
    }

    #[rstest]
    fn pathfinding_neighbour(mut app: App) {
        app.world
            .spawn((Start(HexCoord::new(2, 1)), Goal(HexCoord::new(2, 0))));
        app.add_system(find_path_system);

        app.update();

        let path = app.world.query::<&Path>().single(&app.world);
        println!("path {:?}", path.0);
        assert_eq!(path.1, 1);
    }

    #[rstest]
    fn pathfinding(mut app: App) {
        app.world
            .spawn((Start(HexCoord::new(0, 0)), Goal(HexCoord::new(0, 2))));
        app.add_system(find_path_system);

        app.update();

        let path = app.world.query::<&Path>().single(&app.world);
        println!("path {:?}", path.0);
        assert_eq!(path.1, 5);
    }
}
