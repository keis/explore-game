use super::codex_buffer::{CodexBuffer, CodexBufferPlugin, CodexBufferValue};
use crate::terrain::Decoration;
use bevy::{prelude::*, render::render_resource::*};
use expl_codex::Id;
use expl_map::Fog;

pub type DecorationBuffer = CodexBuffer<DecorationData>;

#[derive(Default)]
pub struct DecorationMaterialPlugin;

impl Plugin for DecorationMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CodexBufferPlugin::<DecorationData>::default(),
            MaterialPlugin::<DecorationMaterial>::default(),
        ))
        .add_systems(Update, apply_to_material);
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct DecorationData {
    color_a: Vec4,
    color_b: Vec4,
    color_c: Vec4,
}

impl CodexBufferValue for DecorationData {
    type CodexValue = Decoration;
}

impl From<&Decoration> for DecorationData {
    fn from(value: &Decoration) -> Self {
        Self {
            color_a: value.color_a.to_linear().to_f32_array().into(),
            color_b: value.color_b.to_linear().to_f32_array().into(),
            color_c: value.color_c.to_linear().to_f32_array().into(),
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct TerrainFlags: u32 {
        const VISIBLE = 1;
        const EXPLORED = 2;
    }
}

impl From<&Fog> for TerrainFlags {
    fn from(value: &Fog) -> Self {
        let mut flags = TerrainFlags::empty();
        flags.set(TerrainFlags::VISIBLE, value.visible);
        flags.set(TerrainFlags::EXPLORED, value.explored);
        flags
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct DecorationMaterialUniform {
    decoration_idx: u32,
    flags: u32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct DecorationMaterial {
    #[storage(0, read_only, buffer)]
    decoration_data: Buffer,
    #[uniform(1)]
    uniform: DecorationMaterialUniform,
}

impl DecorationMaterial {
    pub fn new(
        decoration_id: &Id<Decoration>,
        fog: &Fog,
        decoration_buffer: &Res<DecorationBuffer>,
    ) -> Self {
        let decoration_idx = decoration_buffer.as_index(decoration_id).unwrap() as u32;
        Self {
            decoration_data: decoration_buffer.buffer().unwrap().clone(),
            uniform: DecorationMaterialUniform {
                decoration_idx,
                flags: TerrainFlags::from(fog).bits(),
            },
        }
    }

    fn set_visible(&mut self, visible: bool) {
        let mut flags = TerrainFlags::from_bits_retain(self.uniform.flags);
        flags.set(TerrainFlags::VISIBLE, visible);
        self.uniform.flags = flags.bits();
    }

    fn set_explored(&mut self, explored: bool) {
        let mut flags = TerrainFlags::from_bits_retain(self.uniform.flags);
        flags.set(TerrainFlags::EXPLORED, explored);
        self.uniform.flags = flags.bits();
    }
}

impl Material for DecorationMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/decoration_fragment.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut terrain_materials: ResMut<Assets<DecorationMaterial>>,
    terrain_query: Query<(&Fog, &MeshMaterial3d<DecorationMaterial>), Changed<Fog>>,
) {
    for (fog, handle) in &terrain_query {
        let Some(material) = terrain_materials.get_mut(handle) else {
            continue;
        };
        material.set_visible(fog.visible);
        material.set_explored(fog.explored);
    }
}
