use crate::{
    map::{HexCoord, ZoneLayer},
    terrain::Terrain,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use pathfinding::prelude::astar;

#[derive(SystemParam)]
pub struct PathFinder<'w, 's> {
    map_query: Query<'w, 's, &'static ZoneLayer>,
    terrain_query: Query<'w, 's, &'static Terrain>,
}

impl<'w, 's> PathFinder<'w, 's> {
    pub fn find_path(&self, start: HexCoord, goal: HexCoord) -> Option<(Vec<HexCoord>, u32)> {
        let zone_layer = self.map_query.single();
        astar(
            &start,
            |p| {
                p.neighbours()
                    .filter(&|c: &HexCoord| {
                        zone_layer
                            .get(*c)
                            .and_then(|&entity| self.terrain_query.get(entity).ok())
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

#[cfg(test)]
mod tests {
    use super::{HexCoord, PathFinder};
    use crate::map::tests::app;
    use bevy::prelude::*;
    use rstest::*;

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
        app.add_systems(Update, find_path_system);

        app.update();

        let path = app.world.query::<&Path>().single(&app.world);
        println!("path {:?}", path.0);
        assert_eq!(path.1, 1);
    }

    #[rstest]
    fn pathfinding(mut app: App) {
        app.world
            .spawn((Start(HexCoord::new(0, 0)), Goal(HexCoord::new(0, 2))));
        app.add_systems(Update, find_path_system);

        app.update();

        let path = app.world.query::<&Path>().single(&app.world);
        println!("path {:?}", path.0);
        assert_eq!(path.1, 5);
    }
}
