use super::{asset::*, component::*, system_param::structure_codex};
use crate::{
    actor::Actor, creature::Creature, floating_text::FloatingTextSource, input::Selection,
    inventory::Inventory, material::StructureMaterialTemplate, role::Role, ExplError,
};
use bevy::{
    prelude::*,
    scene::{bsn, EntityWorldMutSceneExt},
};
use bevy_mod_outline::OutlineVolume;
use expl_codex::Id;
use expl_map::{Fog, FogVisibility, HexCoord, MapPresence};

/// Role presenting a structure.
///
/// Only the [`Id`] is stored; the codex entry is resolved when the role is
/// attached, while placement comes from [`MapPresence`].
pub struct StructureRole {
    structure_id: Id<Structure>,
}

impl StructureRole {
    pub fn new(structure_id: Id<Structure>) -> Self {
        Self { structure_id }
    }

    /// Builds the presentation scene owned by the role.
    ///
    /// Takes `self` because [`Scene`] is `'static`: the scene must own every
    /// value it is built from.
    fn scene(self, structure: &Structure) -> impl Scene {
        let Self { structure_id } = self;
        let mesh = structure.mesh.clone();

        bsn! {
            ~FogVisibility
            Selection
            FloatingTextSource { offset: Vec3::new(0.0, 0.5, 0.0) }
            Children [
                Transform {
                    translation: Vec3::ZERO,
                    scale: Vec3::splat(structure.scale),
                    rotation: Quat::from_rotation_y(structure.rotation)
                }
                Mesh3d(mesh)
                ~StructureMaterialTemplate { structure_id }
                OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: { structure.color_a }
                }
            ]
        }
    }
}

impl Role for StructureRole {
    fn attach(self, entity: &mut EntityWorldMut) -> Result<(), ExplError> {
        let structure = structure_codex(entity.world())?[&self.structure_id].clone();
        entity.apply_scene(self.scene(&structure))?;
        Ok(())
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

    pub fn with_fluff(self) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(*self.structure_id);
        (self, structure_role)
    }
}

#[derive(Bundle, Default)]
pub struct CampBundle {
    structure_id: StructureId,
    camp: Camp,
    inventory: Inventory,
    presence: MapPresence,
    fog: Fog,
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
        }
    }

    pub fn with_fluff(self) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(*self.structure_id);
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

    pub fn with_fluff(self) -> (Self, StructureRole) {
        let structure_role = StructureRole::new(*self.structure_id);
        (self, structure_role)
    }
}

#[derive(Bundle, Default)]
pub struct SafeHavenBundle {
    safe_haven: SafeHaven,
    inventory: Inventory,
}
