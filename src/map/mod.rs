use bevy::{ecs::schedule::ShouldRun, prelude::*};
use pathfinding::prelude::astar;

pub mod events;
mod hexcoord;
mod layout;
mod pathguided;
mod presence;

pub use hexcoord::HexCoord;
pub use layout::{MapLayout, MapLayoutIterator};
pub use pathguided::PathGuided;
pub use presence::MapPresence;

pub struct Map {
    tiles: Vec<Option<Entity>>,
    pub layout: MapLayout,
}

impl Map {
    pub fn new(layout: MapLayout) -> Self {
        Self {
            layout,
            tiles: vec![None; layout.size()],
        }
    }

    pub fn set(&mut self, position: HexCoord, entity: Option<Entity>) {
        if let Some(offset) = self.layout.offset(position) {
            self.tiles[offset] = entity;
        }
    }

    pub fn get(&self, position: HexCoord) -> Option<Entity> {
        self.layout
            .offset(position)
            .and_then(|offset| self.tiles[offset])
    }
}

#[derive(Component)]
pub struct MapComponent {
    pub map: Map,
    pub radius: f32,
}

pub struct Damaged(bool);

fn run_if_damaged(damaged: Res<Damaged>) -> ShouldRun {
    if damaged.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn damage(mut entered_event: EventReader<events::Entered>, mut damaged: ResMut<Damaged>) {
    for _event in entered_event.iter() {
        damaged.0 = true;
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Damaged(true))
            .add_system(pathguided::progress_path_guided)
            .add_system(pathguided::reset_movement_points)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_damaged)
                    .with_system(presence::update_visibility),
            )
            .add_system_to_stage(CoreStage::PostUpdate, damage)
            .add_event::<events::Entered>();
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
                .into_iter()
                .filter(is_walkable)
                .map(|p| (p, 1))
                .collect::<Vec<(HexCoord, u32)>>()
        },
        |p| p.distance(&goal).try_into().unwrap(),
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
        println!("neigbours {:?}", start.neighbours());
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 1);
    }

    #[test]
    fn pathfinding() {
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new(4, 2);

        let result = find_path(start, goal, &|_| true);
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 6);
    }
}