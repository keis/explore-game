use crate::{
    assets::MainAssets,
    map::{Fog, HexCoord, MapPresence},
    material::{PortalMaterial, TerrainMaterial},
    terrain::HeightQuery,
};
use bevy::{pbr::NotShadowCaster, prelude::*};

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Portal {
    pub open: bool,
}

pub type PortalParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<TerrainMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct PortalBundle {
    presence: MapPresence,
    fog: Fog,
    portal: Portal,
}

#[derive(Bundle, Default)]
pub struct PortalFluffBundle {
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl PortalBundle {
    pub fn new(position: HexCoord) -> Self {
        Self {
            presence: MapPresence { position },
            ..default()
        }
    }

    pub fn with_fluff(self, portal_params: &mut PortalParams) -> (Self, PortalFluffBundle) {
        let fluff = PortalFluffBundle::new(portal_params, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl PortalFluffBundle {
    pub fn new(
        (main_assets, terrain_materials, height_query): &mut PortalParams,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.4, 0.42, 0.4),
                    ..default()
                }),
                visibility: if fog.visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                )
                .with_scale(Vec3::splat(0.3))
                .with_rotation(Quat::from_rotation_y(2.0)),
                ..default()
            },
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_portal(
    mut commands: Commands,
    mut portal_params: PortalParams,
    portal_query: Query<(Entity, &MapPresence, &Fog), (With<Portal>, Without<GlobalTransform>)>,
) {
    for (entity, presence, fog) in &portal_query {
        commands
            .entity(entity)
            .insert(PortalFluffBundle::new(&mut portal_params, presence, fog));
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
