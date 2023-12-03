use super::component::*;
use crate::{
    actor::Group,
    assets::MainAssets,
    floating_text::FloatingTextSource,
    input::SelectionBundle,
    inventory::Inventory,
    map::{Fog, FogRevealer, HexCoord, MapPresence, Offset, ViewRadius},
    material::TerrainMaterial,
    terrain::HeightQuery,
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
                    visible: fog.visible,
                    explored: fog.explored,
                    ..default()
                }),
                visibility: if fog.explored {
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
    inventory: Inventory,
    presence: MapPresence,
    group: Group,
    offset: Offset,
    view_radius: ViewRadius,
    fog: Fog,
    fog_revealer: FogRevealer,
}

#[derive(Bundle, Default)]
pub struct CampFluffBundle {
    selection: SelectionBundle,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
    floating_text_source: FloatingTextSource,
}

impl CampBundle {
    pub fn new(position: HexCoord, name: String, inventory: Inventory) -> Self {
        Self {
            camp: Camp { name },
            presence: MapPresence { position },
            inventory,
            ..default()
        }
    }

    pub fn with_fluff(self, camp_params: &mut CampParams) -> (Self, CampFluffBundle) {
        let fluff = CampFluffBundle::new(camp_params, &self.presence, &self.offset, &self.fog);
        (self, fluff)
    }
}

impl CampFluffBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut CampParams,
        presence: &MapPresence,
        offset: &Offset,
        fog: &Fog,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.tent_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.631, 0.596, 0.165),
                    visible: fog.visible,
                    explored: fog.explored,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::from(presence.position) + offset.0)
                    .with_rotation(Quat::from_rotation_y(1.0))
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            },
            floating_text_source: FloatingTextSource::with_offset(Vec3::new(0.0, 0.5, 0.0)),
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
                    visible: fog.visible,
                    explored: fog.explored,
                    ..default()
                }),
                visibility: if fog.explored {
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

#[derive(Bundle, Default)]
pub struct SafeHavenBundle {
    safe_haven: SafeHaven,
    inventory: Inventory,
    group: Group,
}
