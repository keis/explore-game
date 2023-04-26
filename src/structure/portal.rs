use crate::{
    assets::MainAssets,
    map::{Fog, HexCoord},
    material::TerrainMaterial,
};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Portal;

#[derive(Bundle, Default)]
pub struct PortalBundle {
    fog: Fog,
    portal: Portal,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

pub type PortalParams<'w, 's> = (Res<'w, MainAssets>, ResMut<'w, Assets<TerrainMaterial>>);

impl PortalBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut PortalParams,
        position: HexCoord,
        height: f32,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.4, 0.42, 0.4),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(
                    Vec3::from(position) + Vec3::new(0.0, height, 0.0),
                )
                .with_scale(Vec3::splat(0.3))
                .with_rotation(Quat::from_rotation_y(2.0)),
                ..default()
            },
            ..default()
        }
    }
}
