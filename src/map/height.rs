use super::{HexCoord, ZoneLayer};
use bevy::{ecs::system::SystemParam, prelude::*};
use glam::Vec3Swizzles;
use noisy_bevy::simplex_noise_2d;

const HEX_RADIUS_RATIO: f32 = 0.866_025_4;

#[derive(Component, Default, Copy, Clone)]
pub struct Height {
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
}

impl Height {
    pub fn height_at(&self, local_position: Vec2, world_position: Vec2) -> f32 {
        let dc = local_position.length();
        let da = (local_position.abs() - Vec2::new(HEX_RADIUS_RATIO, 0.5)).length();
        let db = (local_position.abs() - Vec2::new(0.0, 1.0)).length();
        let (amp, base) = if dc < 0.7 {
            (self.height_amp, self.height_base)
        } else if da < db {
            if local_position.x > 0.0 {
                if local_position.y > 0.0 {
                    (self.outer_amp[0], self.outer_base[0])
                } else {
                    (self.outer_amp[5], self.outer_base[5])
                }
            } else if local_position.y > 0.0 {
                (self.outer_amp[2], self.outer_base[2])
            } else {
                (self.outer_amp[3], self.outer_base[3])
            }
        } else if local_position.y > 0.0 {
            (self.outer_amp[1], self.outer_base[1])
        } else {
            (self.outer_amp[4], self.outer_base[4])
        };
        let noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
        base + noise * amp
    }
}

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
