use super::component::*;
use crate::{
    actor::Group,
    assets::MainAssets,
    input::SelectionBundle,
    map::{Fog, HexCoord, MapPresence, Offset, ViewRadius},
    material::TerrainMaterial,
    terrain::HeightQuery,
    VIEW_RADIUS,
};
use bevy::prelude::*;

pub type SpawnerParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<TerrainMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct SpawnerBundle {
    presence: MapPresence,
    fog: Fog,
    spawner: Spawner,
}

#[derive(Bundle, Default)]
pub struct SpawnerFluffBundle {
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl SpawnerBundle {
    pub fn new(position: HexCoord) -> Self {
        Self {
            presence: MapPresence { position },
            ..default()
        }
    }

    pub fn with_fluff(self, spawner_params: &mut SpawnerParams) -> (Self, SpawnerFluffBundle) {
        let fluff = SpawnerFluffBundle::new(spawner_params, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl SpawnerFluffBundle {
    pub fn new(
        (main_assets, terrain_materials, height_query): &mut SpawnerParams,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.8, 0.32, 0.3),
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

pub type CampParams<'w> = (Res<'w, MainAssets>, ResMut<'w, Assets<TerrainMaterial>>);

#[derive(Bundle, Default)]
pub struct CampBundle {
    camp: Camp,
    presence: MapPresence,
    group: Group,
    offset: Offset,
    view_radius: ViewRadius,
    fog: Fog,
}

#[derive(Bundle, Default)]
pub struct CampFluffBundle {
    selection: SelectionBundle,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl CampBundle {
    pub fn new(position: HexCoord, camp: Camp) -> Self {
        Self {
            camp,
            presence: MapPresence { position },
            view_radius: ViewRadius(VIEW_RADIUS),
            ..default()
        }
    }

    pub fn with_fluff(self, camp_params: &mut CampParams) -> (Self, CampFluffBundle) {
        let fluff = CampFluffBundle::new(camp_params, &self.presence, &self.offset);
        (self, fluff)
    }
}

impl CampFluffBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut CampParams,
        presence: &MapPresence,
        offset: &Offset,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.tent_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.631, 0.596, 0.165),
                    visible: true,
                    explored: true,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::from(presence.position) + offset.0)
                    .with_rotation(Quat::from_rotation_y(1.0))
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            },
            ..default()
        }
    }
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
