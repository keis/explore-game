use super::{
    asset::Terrain,
    component::{Height, OuterTerrain, TerrainId},
};
use crate::codex::CodexParam;
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_map::{HexCoord, ZoneLayer};
use glam::Vec3Swizzles;

#[derive(SystemParam)]
pub struct HeightQuery<'w, 's> {
    terrain_codex: CodexParam<'w, Terrain>,
    map_query: Query<'w, 's, &'static ZoneLayer>,
    terrain_query: Query<'w, 's, (&'static TerrainId, &'static OuterTerrain)>,
}

impl HeightQuery<'_, '_> {
    pub fn get(&self, point: Vec3) -> f32 {
        let terrain_codex = self.terrain_codex.get().unwrap();
        let zone_layer = self.map_query.single();
        let coord: HexCoord = point.into();
        zone_layer
            .ok()
            .and_then(|zone_layer| zone_layer.get(coord))
            .and_then(|&entity| self.terrain_query.get(entity).ok())
            .map_or(0.0, |(terrain_id, outer_terrain)| {
                let height = Height::new(terrain_codex, **terrain_id, outer_terrain);
                height.height_at((point - Vec3::from(coord)).xz(), point.xz())
            })
    }

    pub fn adjust(&self, point: Vec3) -> Vec3 {
        Vec3::new(point.x, self.get(point), point.z)
    }
}
