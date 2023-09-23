use super::component::Height;
use crate::map::{HexCoord, ZoneLayer};
use bevy::{ecs::system::SystemParam, prelude::*};
use glam::Vec3Swizzles;

#[derive(SystemParam)]
pub struct HeightQuery<'w, 's> {
    map_query: Query<'w, 's, &'static ZoneLayer>,
    height_query: Query<'w, 's, &'static Height>,
}

impl<'w, 's> HeightQuery<'w, 's> {
    pub fn get(&self, point: Vec3) -> f32 {
        let zone_layer = self.map_query.single();
        let coord: HexCoord = point.into();
        zone_layer
            .get(coord)
            .and_then(|&entity| self.height_query.get(entity).ok())
            .map_or(0.0, |height| {
                height.height_at((point - Vec3::from(coord)).xz(), point.xz())
            })
    }

    pub fn adjust(&self, point: Vec3) -> Vec3 {
        Vec3::new(point.x, self.get(point), point.z)
    }
}
