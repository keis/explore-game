use super::Fog;
use crate::{assets::MainAssets, material::TerrainMaterial};
use bevy::prelude::*;

#[derive(Component)]
pub struct ZoneDecorationCrystals;

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
        translation: Vec3,
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
                transform: Transform::from_translation(translation)
                    .with_scale(Vec3::splat(scale * 0.3)),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct ZoneDecorationTree;

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
        translation: Vec3,
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
                transform: Transform::from_translation(translation)
                    .with_scale(Vec3::splat(scale * 0.5)),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct ZoneDecorationPortal;

#[derive(Bundle)]
pub struct ZoneDecorationPortalBundle {
    fog: Fog,
    zone_decoration_portal: ZoneDecorationPortal,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl ZoneDecorationPortalBundle {
    pub fn new(
        main_assets: &Res<MainAssets>,
        terrain_materials: &mut ResMut<Assets<TerrainMaterial>>,
        translation: Vec3,
    ) -> Self {
        Self {
            fog: Fog::default(),
            zone_decoration_portal: ZoneDecorationPortal,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.4, 0.42, 0.4),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(translation)
                    .with_scale(Vec3::splat(0.3))
                    .with_rotation(Quat::from_rotation_y(10.0)),
                ..default()
            },
        }
    }
}
