use super::component::*;
use crate::{
    assets::MainAssets,
    map::{Fog, HexAssets, HexCoord},
    material::{TerrainMaterial, WaterMaterial},
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use glam::Vec3Swizzles;

#[derive(Bundle)]
pub struct ZoneDecorationCrystalsBundle {
    fog: Fog,
    zone_decoration_crystals: ZoneDecorationCrystals,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl ZoneDecorationCrystalsBundle {
    pub fn new(
        main_assets: &Res<MainAssets>,
        terrain_materials: &mut ResMut<Assets<TerrainMaterial>>,
        height: &Height,
        position: HexCoord,
        relative: Vec2,
        scale: f32,
    ) -> Self {
        Self {
            fog: Fog::default(),
            zone_decoration_crystals: ZoneDecorationCrystals,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.crystals_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgba(0.7, 0.4, 0.4, 0.777),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(Vec3::new(
                    relative.x,
                    height.height_at(relative, Vec3::from(position).xz() + relative),
                    relative.y,
                ))
                .with_scale(Vec3::splat(scale * 0.3)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ZoneDecorationTreeBundle {
    fog: Fog,
    zone_decoration_tree: ZoneDecorationTree,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl ZoneDecorationTreeBundle {
    pub fn new(
        main_assets: &Res<MainAssets>,
        terrain_materials: &mut ResMut<Assets<TerrainMaterial>>,
        height: &Height,
        position: HexCoord,
        relative: Vec2,
        scale: f32,
    ) -> Self {
        Self {
            fog: Fog::default(),
            zone_decoration_tree: ZoneDecorationTree,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.pine_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    texture: Some(main_assets.forest_texture.clone()),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(Vec3::new(
                    relative.x,
                    height.height_at(relative, Vec3::from(position).xz() + relative),
                    relative.y,
                ))
                .with_scale(Vec3::splat(scale * 0.5)),
                ..default()
            },
        }
    }
}

#[derive(Bundle, Default)]
pub struct WaterBundle {
    water: Water,
    fog: Fog,
    not_shadow_caster: NotShadowCaster,
    material_mesh_bundle: MaterialMeshBundle<WaterMaterial>,
}

impl WaterBundle {
    pub fn new(
        hex_assets: &Res<HexAssets>,
        water_materials: &mut ResMut<Assets<WaterMaterial>>,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: water_materials.add(WaterMaterial {
                    color: Color::rgba(0.1, 0.1, 0.8, 0.4),
                }),
                transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
                ..default()
            },
            ..default()
        }
    }
}
