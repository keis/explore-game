use crate::hexgrid::layout::SquareGridLayout;
use crate::hexgrid::{Grid, GridLayout};
use crate::State;
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use pathfinding::prelude::astar;
use std::collections::hash_set::HashSet;

mod commands;
mod events;
mod fog;
mod generator;
mod pathdisplay;
mod pathguided;
mod position;
mod presence;
mod zone;

pub use crate::hexgrid::HexCoord;
pub use commands::{AddMapPresence, DespawnPresence, MoveMapPresence};
pub use events::MapEvent;
pub use fog::Fog;
pub use generator::{start_map_generation, GenerateMapTask};
pub use pathdisplay::PathDisplay;
pub use pathguided::PathGuided;
pub use position::MapPosition;
pub use presence::{MapPresence, Offset, ViewRadius};
pub use zone::{Terrain, Zone, ZoneBundle};

#[derive(Component)]
pub struct GameMap {
    tiles: Grid<SquareGridLayout, Entity>,
    presence: Grid<SquareGridLayout, HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl GameMap {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>) -> Self {
        GameMap {
            tiles: Grid {
                layout,
                data: tiles,
            },
            presence: Grid {
                layout,
                data: vec![HashSet::new(); layout.size()],
            },
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

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Damaged(true))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_damaged)
                    .with_system(presence::update_visibility),
            )
            .add_system_set(
                SystemSet::on_update(State::Running).with_system(pathdisplay::update_path_display),
            )
            .add_system_to_stage(CoreStage::PostUpdate, damage)
            .add_event::<MapEvent>();
    }
}

pub fn find_path(
    map: &GameMap,
    zone_query: &Query<&Zone>,
    start: HexCoord,
    goal: HexCoord,
) -> Option<(Vec<HexCoord>, u32)> {
    astar(
        &start,
        |p| {
            p.neighbours()
                .filter(&|c: &HexCoord| {
                    map.get(*c)
                        .and_then(|&entity| zone_query.get(entity).ok())
                        .map_or(false, |zone| zone.is_walkable())
                })
                .map(|p| (p, 1))
                .collect::<Vec<(HexCoord, u32)>>()
        },
        |p| p.distance(&goal),
        |p| *p == goal,
    )
}

pub fn spawn_game_map_from_prototype<F>(
    commands: &mut Commands,
    prototype: &Grid<SquareGridLayout, Terrain>,
    mut spawn_tile: F,
) -> Entity
where
    F: FnMut(&mut Commands, HexCoord, Terrain) -> Entity,
{
    let gamemap = GameMap {
        tiles: Grid {
            layout: prototype.layout,
            data: prototype
                .layout
                .iter()
                .map(|coord| spawn_tile(commands, coord, prototype[coord]))
                .collect(),
        },
        presence: Grid {
            layout: prototype.layout,
            data: vec![HashSet::new(); prototype.layout.size()],
        },
        void: HashSet::new(),
    };
    commands.spawn(gamemap).id()
}

#[cfg(test)]
mod tests {
    use super::{find_path, GameMap, HexCoord, SquareGridLayout, Terrain, Zone};
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
        map_query: Query<&GameMap>,
        zone_query: Query<&Zone>,
        params_query: Query<(Entity, &Start, &Goal)>,
    ) {
        let map = map_query.single();
        let (entity, start, goal) = params_query.single();
        if let Some(path) = find_path(map, &zone_query, start.0, goal.0) {
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
