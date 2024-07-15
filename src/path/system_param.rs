use crate::{
    map::{HexCoord, ZoneLayer},
    terrain::{Terrain, TerrainCodex, TerrainId},
    ExplError,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_codex::Codex;
use pathfinding::prelude::astar;

#[derive(SystemParam)]
pub struct PathFinder<'w, 's> {
    map_query: Query<'w, 's, &'static ZoneLayer>,
    terrain_query: Query<'w, 's, &'static TerrainId>,
    terrain_codex: TerrainCodex<'w>,
}

impl<'w, 's> PathFinder<'w, 's> {
    pub fn get(&self) -> Result<BoundPathFinder, ExplError> {
        Ok(BoundPathFinder {
            zone_layer: self.map_query.get_single()?,
            terrain_codex: self.terrain_codex.get()?,
            terrain_query: &self.terrain_query,
        })
    }
}

pub struct BoundPathFinder<'w, 's> {
    zone_layer: &'s ZoneLayer,
    terrain_codex: &'s Codex<Terrain>,
    terrain_query: &'s Query<'w, 's, &'static TerrainId>,
}

impl<'w, 's> BoundPathFinder<'w, 's> {
    pub fn find_path(&self, start: HexCoord, goal: HexCoord) -> Option<Vec<(HexCoord, Entity)>> {
        let start = self.zone_layer.get(start).map(|e| (start, *e)).unwrap();
        astar(
            &start,
            |(p, _)| {
                p.neighbours()
                    .filter_map(|p| self.zone_layer.get(p).map(|e| (p, *e)))
                    .filter(|(_, e)| self.is_walkable(*e))
                    .map(|r| (r, 1))
                    .collect::<Vec<((HexCoord, Entity), u32)>>()
            },
            |(p, _)| p.distance(goal),
            |(p, _)| *p == goal,
        )
        .map(|(path, _len)| path)
    }

    pub fn is_walkable(&self, entity: Entity) -> bool {
        self.terrain_query
            .get(entity)
            .ok()
            .map_or(false, |terrain_id| {
                self.terrain_codex[terrain_id].allow_walking
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{HexCoord, PathFinder};
    use crate::test_fixture::app;
    use bevy::prelude::*;
    use rstest::*;

    #[derive(Component, Debug)]
    struct Goal(HexCoord);

    #[derive(Component, Debug)]
    struct Start(HexCoord);

    #[derive(Component, Debug)]
    struct Path(Vec<(HexCoord, Entity)>);

    fn find_path_system(
        mut commands: Commands,
        path_finder: PathFinder,
        params_query: Query<(Entity, &Start, &Goal)>,
    ) {
        let (entity, start, goal) = params_query.single();
        if let Some(path) = path_finder.get().unwrap().find_path(start.0, goal.0) {
            commands.entity(entity).insert(Path(path));
        } else {
            println!("WHAT");
        }
    }

    #[rstest]
    fn pathfinding_neighbour(mut app: App) {
        app.world_mut()
            .spawn((Start(HexCoord::new(2, 1)), Goal(HexCoord::new(2, 0))));
        app.add_systems(Update, find_path_system);

        app.update();

        let path = app.world_mut().query::<&Path>().single(app.world());
        println!("path {:?}", path.0);
        assert_eq!(path.0.len(), 2);
    }

    #[rstest]
    fn pathfinding(mut app: App) {
        app.world_mut()
            .spawn((Start(HexCoord::new(0, 0)), Goal(HexCoord::new(0, 2))));
        app.add_systems(Update, find_path_system);

        app.update();

        let path = app.world_mut().query::<&Path>().single(app.world());
        println!("path {:?}", path.0);
        assert_eq!(path.0.len(), 6);
    }
}
