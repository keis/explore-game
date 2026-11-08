#![allow(dead_code)]
use super::codex_buffer::{CodexBuffer, CodexBufferPlugin, CodexBufferValue};
use crate::structure::{Structure, StructureId};
use bevy::{
    ecs::template::{Template, TemplateContext},
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};
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
    pub fn from_structure(structure_id: Id<Structure>, structure_buffer: &StructureBuffer) -> Self {
        let structure_idx = structure_buffer.as_index(&structure_id).unwrap() as u32;
        Self {
            structure_data: structure_buffer.buffer().unwrap().clone(),
            uniform: StructureMaterialUniform {
                structure_idx,
                flags: StructureFlags::empty().bits(),
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

/// Scene template building the [`StructureMaterial`] for a [`StructureId`].
#[derive(Default)]
pub struct StructureMaterialTemplate {
    pub structure_id: Id<Structure>,
}

impl Template for StructureMaterialTemplate {
    type Output = MeshMaterial3d<StructureMaterial>;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Self::Output> {
        let material = StructureMaterial::from_structure(
            self.structure_id,
            context.resource::<StructureBuffer>(),
        );
        let handle = context
            .resource_mut::<Assets<StructureMaterial>>()
            .add(material);
        Ok(MeshMaterial3d(handle))
    }

    fn clone_template(&self) -> Self {
        Self {
            structure_id: self.structure_id,
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut structure_materials: ResMut<Assets<StructureMaterial>>,
    structure_query: Query<(&Fog, &Children), (Changed<Fog>, With<StructureId>)>,
    material_query: Query<&MeshMaterial3d<StructureMaterial>>,
) {
    for (fog, children) in &structure_query {
        let mut materials = material_query.iter_many(children);
        while let Some(handle) = materials.fetch_next() {
            let Some(mut material) = structure_materials.get_mut(handle) else {
                continue;
            };
            material.set_visible(fog.visible);
            material.set_explored(fog.explored);
        }
    }
}
