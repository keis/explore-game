use crate::{
    assets::MainAssets,
    map::{Fog, HeightQuery, HexCoord},
    material::{PortalMaterial, TerrainMaterial},
};
use bevy::{pbr::NotShadowCaster, prelude::*};

#[derive(Component, Default, Debug)]
pub struct Portal {
    pub open: bool,
}

#[derive(Bundle, Default)]
pub struct PortalBundle {
    fog: Fog,
    portal: Portal,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

pub type PortalParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<TerrainMaterial>>,
    HeightQuery<'w, 's>,
);

impl PortalBundle {
    pub fn new(
        (main_assets, terrain_materials, height_query): &mut PortalParams,
        position: HexCoord,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.4, 0.42, 0.4),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(height_query.adjust(position.into()))
                    .with_scale(Vec3::splat(0.3))
                    .with_rotation(Quat::from_rotation_y(2.0)),
                ..default()
            },
            ..default()
        }
    }
}

pub fn update_portal_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut portal_materials: ResMut<Assets<PortalMaterial>>,
    portal_query: Query<(Entity, &Portal), Changed<Portal>>,
) {
    for (entity, portal) in &portal_query {
        if portal.open {
            commands
                .spawn((
                    NotShadowCaster,
                    MaterialMeshBundle {
                        mesh: meshes.add(shape::Plane::from_size(2.0).into()),
                        material: portal_materials.add(PortalMaterial {
                            base_color: Color::rgba(0.2, 0.7, 0.1, 0.3),
                            swirl_color: Color::rgba(0.4, 0.2, 0.7, 0.7),
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.9, 0.0))
                            .with_rotation(
                                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                                    * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                            ),
                        ..default()
                    },
                ))
                .set_parent(entity);
        }
    }
}
