use super::{asset::*, component::*};
use crate::{
    actor::{Actor, Members},
    creature::Creature,
    floating_text::FloatingTextSource,
    input::{DefaultOutlineVolume, SelectionBundle},
    inventory::Inventory,
    material::{StructureBuffer, StructureMaterial},
    role::Role,
    terrain::HeightQuery,
};
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use expl_codex::{Codex, Id};
use expl_map::{Fog, FogRevealer, HexCoord, MapPresence, ViewRadius};

pub type StructureParams<'w, 's> = (
    Res<'w, StructureBuffer>,
    HeightQuery<'w, 's>,
    ResMut<'w, Assets<StructureMaterial>>,
);

#[derive(Default)]
pub struct StructureRole {
    // Insert
    spatial_bundle: SpatialBundle,
    selection: SelectionBundle,
    floating_text_source: FloatingTextSource,
    // Child
    material_mesh_bundle: MaterialMeshBundle<StructureMaterial>,
    default_outline_volume: DefaultOutlineVolume,
    outline_bundle: OutlineBundle,
}

impl StructureRole {
    pub fn new(
        (structure_buffer, height_query, structure_materials): &mut StructureParams,
        structure_codex: &Codex<Structure>,
        structure_id: Id<Structure>,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        let structure = &structure_codex[&structure_id];
        let outline = OutlineVolume {
            visible: true,
            width: 2.0,
            colour: structure.color_a,
        };
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
            material_mesh_bundle: MaterialMeshBundle {
                mesh: structure.mesh.clone(),
                material: structure_materials.add(StructureMaterial::from_structure(
                    &structure_id,
                    fog,
                    structure_buffer,
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
            ..default()
        }
    }
}

impl Role for StructureRole {
    fn attach(self, entity: &mut EntityWorldMut) {
        entity
            .insert((
                self.spatial_bundle,
                self.selection,
                self.floating_text_source,
            ))
            .with_children(|parent| {
                parent.spawn((
                    self.material_mesh_bundle,
                    self.default_outline_volume,
                    self.outline_bundle,
                ));
            });
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
    pub fn new(position: HexCoord, creature: Id<Creature>, actor: Id<Actor>) -> Self {
        Self {
            structure_id: StructureId::from_tag("spawner"),
            presence: MapPresence { position },
            spawner: Spawner {
                creature,
                actor,
                ..default()
            },
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        structure_params: &mut StructureParams,
        structure_codex: &Codex<Structure>,
    ) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        (self, structure_role)
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
    ) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        (self, structure_role)
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
    ) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(
            structure_params,
            structure_codex,
            *self.structure_id,
            &self.presence,
            &self.fog,
        );
        (self, structure_role)
    }
}

#[derive(Bundle, Default)]
pub struct SafeHavenBundle {
    safe_haven: SafeHaven,
    inventory: Inventory,
    members: Members,
}
