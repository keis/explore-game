use super::{asset::*, component::*};
use crate::{
    actor::Group,
    floating_text::FloatingTextSource,
    input::SelectionBundle,
    inventory::Inventory,
    map::{Fog, FogRevealer, HexCoord, MapPresence, ViewRadius},
    material::TerrainMaterial,
    terrain::HeightQuery,
};
use bevy::prelude::*;

pub type SpawnerParams<'w, 's> = (ResMut<'w, Assets<TerrainMaterial>>, HeightQuery<'w, 's>);

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

    pub fn with_fluff(
        self,
        spawner_params: &mut SpawnerParams,
        structure_codex: &Codex<Structure>,
    ) -> (Self, SpawnerFluffBundle) {
        let fluff =
            SpawnerFluffBundle::new(spawner_params, structure_codex, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl SpawnerFluffBundle {
    pub fn new(
        (terrain_materials, height_query): &mut SpawnerParams,
        structure_codex: &Codex<Structure>,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        let structure_id = Id::from_tag("spawner");
        let structure = &structure_codex[&structure_id];
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: structure.mesh.clone(),
                material: terrain_materials.add(TerrainMaterial::from_structure(
                    structure_codex,
                    &structure_id,
                    fog,
                )),
                visibility: if fog.explored {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                )
                .with_scale(Vec3::splat(structure.scale))
                .with_rotation(Quat::from_rotation_y(structure.rotation)),
                ..default()
            },
        }
    }
}

pub type CampParams<'w, 's> = (ResMut<'w, Assets<TerrainMaterial>>, HeightQuery<'w, 's>);

#[derive(Bundle, Default)]
pub struct CampBundle {
    camp: Camp,
    inventory: Inventory,
    presence: MapPresence,
    group: Group,
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

    pub fn with_fluff(
        self,
        camp_params: &mut CampParams,
        structure_codex: &Codex<Structure>,
    ) -> (Self, CampFluffBundle) {
        let fluff = CampFluffBundle::new(camp_params, structure_codex, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl CampFluffBundle {
    pub fn new(
        (terrain_materials, height_query): &mut CampParams,
        structure_codex: &Codex<Structure>,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        let structure_id = Id::from_tag("camp");
        let structure = &structure_codex[&structure_id];
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: structure.mesh.clone(),
                material: terrain_materials.add(TerrainMaterial::from_structure(
                    structure_codex,
                    &structure_id,
                    fog,
                )),
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                )
                .with_scale(Vec3::splat(structure.scale))
                .with_rotation(Quat::from_rotation_y(structure.rotation)),
                ..default()
            },
            floating_text_source: FloatingTextSource::with_offset(Vec3::new(0.0, 0.5, 0.0)),
            ..default()
        }
    }
}

pub type PortalParams<'w, 's> = (ResMut<'w, Assets<TerrainMaterial>>, HeightQuery<'w, 's>);

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

    pub fn with_fluff(
        self,
        portal_params: &mut PortalParams,
        structure_codex: &Codex<Structure>,
    ) -> (Self, PortalFluffBundle) {
        let fluff =
            PortalFluffBundle::new(portal_params, structure_codex, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl PortalFluffBundle {
    pub fn new(
        (terrain_materials, height_query): &mut PortalParams,
        structure_codex: &Codex<Structure>,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        let structure_id = Id::from_tag("portal");
        let structure = &structure_codex[&structure_id];
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: structure.mesh.clone(),
                material: terrain_materials.add(TerrainMaterial::from_structure(
                    structure_codex,
                    &structure_id,
                    fog,
                )),
                visibility: if fog.explored {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                )
                .with_scale(Vec3::splat(structure.scale))
                .with_rotation(Quat::from_rotation_y(structure.rotation)),
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
