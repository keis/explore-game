use super::codex_buffer::{CodexBuffer, CodexBufferPlugin, CodexBufferValue};
use crate::structure::Structure;
use bevy::{prelude::*, render::render_resource::*};
use expl_codex::Id;
use expl_map::Fog;

pub type StructureBuffer = CodexBuffer<StructureData>;

#[derive(Default)]
pub struct StructureMaterialPlugin;

impl Plugin for StructureMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CodexBufferPlugin::<StructureData>::default(),
            MaterialPlugin::<StructureMaterial>::default(),
        ))
        .add_systems(Update, apply_to_material);
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct StructureData {
    color_a: Vec4,
    color_b: Vec4,
    color_c: Vec4,
}

impl CodexBufferValue for StructureData {
    type CodexValue = Structure;
}

impl From<&Structure> for StructureData {
    fn from(value: &Structure) -> Self {
        Self {
            color_a: value.color_a.to_linear().to_f32_array().into(),
            color_b: value.color_b.to_linear().to_f32_array().into(),
            color_c: value.color_c.to_linear().to_f32_array().into(),
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct StructureFlags: u32 {
        const VISIBLE = 1;
        const EXPLORED = 2;
    }
}

impl From<&Fog> for StructureFlags {
    fn from(value: &Fog) -> Self {
        let mut flags = StructureFlags::empty();
        flags.set(StructureFlags::VISIBLE, value.visible);
        flags.set(StructureFlags::EXPLORED, value.explored);
        flags
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct StructureMaterialUniform {
    structure_idx: u32,
    flags: u32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct StructureMaterial {
    #[storage(0, read_only, buffer)]
    structure_data: Buffer,
    #[uniform(1)]
    uniform: StructureMaterialUniform,
}

impl StructureMaterial {
    pub fn from_structure(
        structure_id: &Id<Structure>,
        fog: &Fog,
        structure_buffer: &Res<StructureBuffer>,
    ) -> Self {
        let structure_idx = structure_buffer.as_index(structure_id).unwrap() as u32;
        Self {
            structure_data: structure_buffer.buffer().unwrap().clone(),
            uniform: StructureMaterialUniform {
                structure_idx,
                flags: StructureFlags::from(fog).bits(),
            },
        }
    }

    fn set_visible(&mut self, visible: bool) {
        let mut flags = StructureFlags::from_bits_retain(self.uniform.flags);
        flags.set(StructureFlags::VISIBLE, visible);
        self.uniform.flags = flags.bits();
    }

    fn set_explored(&mut self, explored: bool) {
        let mut flags = StructureFlags::from_bits_retain(self.uniform.flags);
        flags.set(StructureFlags::EXPLORED, explored);
        self.uniform.flags = flags.bits();
    }
}

impl Material for StructureMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/structure_fragment.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut terrain_materials: ResMut<Assets<StructureMaterial>>,
    terrain_query: Query<(&Fog, &MeshMaterial3d<StructureMaterial>), Changed<Fog>>,
) {
    for (fog, handle) in &terrain_query {
        let Some(material) = terrain_materials.get_mut(handle) else {
            continue;
        };
        material.set_visible(fog.visible);
        material.set_explored(fog.explored);
    }
}
