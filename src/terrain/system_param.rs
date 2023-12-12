use super::{
    asset::{Decoration, Terrain},
    component::Height,
};
use crate::{
    assets::CodexAssets,
    map::{HexCoord, ZoneLayer},
    ExplError,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use expl_codex::Codex;
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

#[derive(SystemParam)]
pub struct TerrainCodex<'w> {
    codex_assets: Res<'w, CodexAssets>,
    terrain_codex_assets: Res<'w, Assets<Codex<Terrain>>>,
}

impl<'w> TerrainCodex<'w> {
    pub fn get(&self) -> Result<&Codex<Terrain>, ExplError> {
        self.terrain_codex_assets
            .get(self.codex_assets.terrain_codex.clone())
            .ok_or(ExplError::MissingCodex)
    }
}

#[derive(SystemParam)]
pub struct DecorationCodex<'w> {
    codex_assets: Res<'w, CodexAssets>,
    decoration_codex_assets: Res<'w, Assets<Codex<Decoration>>>,
}

impl<'w> DecorationCodex<'w> {
    pub fn get(&self) -> Result<&Codex<Decoration>, ExplError> {
        self.decoration_codex_assets
            .get(self.codex_assets.decoration_codex.clone())
            .ok_or(ExplError::MissingCodex)
    }
}
