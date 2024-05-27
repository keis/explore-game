use super::{asset::*, component::*};
use crate::{
    actor::Members,
    creature::Creature,
    floating_text::FloatingTextSource,
    input::{DefaultOutlineVolume, SelectionBundle},
    inventory::Inventory,
    map::{Fog, FogRevealer, HexCoord, MapPresence, ViewRadius},
    material::TerrainMaterial,
    terrain::HeightQuery,
};
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use expl_codex::{Codex, Id};

pub type StructureParams<'w, 's> = (ResMut<'w, Assets<TerrainMaterial>>, HeightQuery<'w, 's>);

#[derive(Bundle, Default)]
pub struct StructureFluffBundle {
    spatial_bundle: SpatialBundle,
    selection: SelectionBundle,
    floating_text_source: FloatingTextSource,
}

#[derive(Bundle, Default)]
pub struct StructureChildBundle {
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
    default_outline_volume: DefaultOutlineVolume,
    outline_bundle: OutlineBundle,
}

impl StructureFluffBundle {
    pub fn new(
        (terrain_materials, height_query): &mut StructureParams,
        structure_codex: &Codex<Structure>,
        structure_id: Id<Structure>,
        presence: &MapPresence,
        fog: &Fog,
    ) -> (Self, StructureChildBundle) {
        let structure = &structure_codex[&structure_id];
        let outline = OutlineVolume {
            visible: true,
            width: 2.0,
            colour: structure.color_a,
        };
        (
            Self {
                floating_text_source: FloatingTextSource::with_offset(Vec3::new(0.0, 0.5, 0.0)),
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(
                        height_query.adjust(presence.position.into()),
                    ),
                    visibility: if fog.explored {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    ..default()
                },
                ..default()
            },
            StructureChildBundle {
                material_mesh_bundle: MaterialMeshBundle {
                    mesh: structure.mesh.clone(),
                    material: terrain_materials.add(TerrainMaterial::from_structure(
                        structure_codex,
                        &structure_id,
                        fog,
                    )),
                    transform: Transform::from_translation(Vec3::ZERO)
                        .with_scale(Vec3::splat(structure.scale))
                        .with_rotation(Quat::from_rotation_y(structure.rotation)),
                    ..default()
                },
                default_outline_volume: DefaultOutlineVolume(outline.clone()),
                outline_bundle: OutlineBundle {
                    outline,
                    ..default()
                },
            },
        )
    }
}

#[derive(Bundle, Default)]
pub struct SpawnerBundle {
    structure_id: StructureId,
    presence: MapPresence,
    fog: Fog,
    spawner: Spawner,
}

impl SpawnerBundle {
    pub fn new(position: HexCoord, creature: Id<Creature>) -> Self {
        Self {
            structure_id: StructureId::from_tag("spawner"),
            presence: MapPresence { position },
            spawner: Spawner {
                creature,
                ..default()
            },
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        structure_params: &mut StructureParams,
        structure_codex: &Codex<Structure>,
    ) -> ((Self, StructureFluffBundle), StructureChildBundle) {
        let (fluff, child) = StructureFluffBundle::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        ((self, fluff), child)
    }
}

#[derive(Bundle, Default)]
pub struct CampBundle {
    structure_id: StructureId,
    camp: Camp,
    inventory: Inventory,
    presence: MapPresence,
    members: Members,
    view_radius: ViewRadius,
    fog: Fog,
    fog_revealer: FogRevealer,
}

impl CampBundle {
    pub fn new(position: HexCoord, name: String, inventory: Inventory) -> Self {
        Self {
            structure_id: StructureId::from_tag("camp"),
            camp: Camp { name },
            presence: MapPresence { position },
            fog: Fog {
                visible: true,
                explored: true,
            },
            inventory,
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        structure_params: &mut StructureParams,
        structure_codex: &Codex<Structure>,
    ) -> ((Self, StructureFluffBundle), StructureChildBundle) {
        let (fluff, child) = StructureFluffBundle::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        ((self, fluff), child)
    }
}

#[derive(Bundle, Default)]
pub struct PortalBundle {
    structure_id: StructureId,
    presence: MapPresence,
    fog: Fog,
    portal: Portal,
}

impl PortalBundle {
    pub fn new(position: HexCoord) -> Self {
        Self {
            structure_id: StructureId::from_tag("portal"),
            presence: MapPresence { position },
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        structure_params: &mut StructureParams,
        structure_codex: &Codex<Structure>,
    ) -> ((Self, StructureFluffBundle), StructureChildBundle) {
        let (fluff, child) = StructureFluffBundle::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        ((self, fluff), child)
    }
}

#[derive(Bundle, Default)]
pub struct SafeHavenBundle {
    safe_haven: SafeHaven,
    inventory: Inventory,
    members: Members,
}
