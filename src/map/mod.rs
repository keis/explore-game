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
    pub radius: f32,
}

impl GameMap {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>, radius: f32) -> Self {
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
            radius,
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
    start: HexCoord,
    goal: HexCoord,
    is_walkable: &impl Fn(&HexCoord) -> bool,
) -> Option<(Vec<HexCoord>, u32)> {
    astar(
        &start,
        |p| {
            p.neighbours()
                .filter(is_walkable)
                .map(|p| (p, 1))
                .collect::<Vec<(HexCoord, u32)>>()
        },
        |p| p.distance(&goal),
        |p| *p == goal,
    )
}

#[cfg(test)]
mod tests {
    use super::{find_path, HexCoord};

    #[test]
    fn pathfinding_neighbour() {
        let start = HexCoord::new(2, 4);
        let goal = HexCoord::new(2, 3);

        let result = find_path(start, goal, &|_| true);
        println!("neigbours {:?}", start.neighbours().collect::<Vec<_>>());
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 1);
    }

    #[test]
    fn pathfinding() {
        let start = HexCoord::ZERO;
        let goal = HexCoord::new(4, 2);

        let result = find_path(start, goal, &|_| true);
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 6);
    }
}
