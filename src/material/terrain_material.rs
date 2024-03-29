use crate::{map::Fog, structure::Structure, terrain::Decoration};
use bevy::{prelude::*, reflect::TypePath, render::render_resource::*};
use expl_codex::{Codex, Id};

#[derive(Default)]
pub struct TerrainMaterialPlugin;

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .add_systems(Update, apply_to_material);
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
#[uniform(0, TerrainMaterialUniform)]
pub struct TerrainMaterial {
    color_a: Color,
    color_b: Color,
    color_c: Color,
    visible: bool,
    explored: bool,
}

impl TerrainMaterial {
    pub fn from_decoration(
        decoration_codex: &Codex<Decoration>,
        decoration_id: &Id<Decoration>,
        fog: &Fog,
    ) -> Self {
        let decoration_data = &decoration_codex[decoration_id];

        Self {
            color_a: decoration_data.color_a,
            color_b: decoration_data.color_b,
            color_c: decoration_data.color_c,
            visible: fog.visible,
            explored: fog.explored,
        }
    }

    pub fn from_structure(
        structure_codex: &Codex<Structure>,
        structure_id: &Id<Structure>,
        fog: &Fog,
    ) -> Self {
        let structure_data = &structure_codex[structure_id];

        Self {
            color_a: structure_data.color_a,
            color_b: structure_data.color_b,
            color_c: structure_data.color_c,
            visible: fog.visible,
            explored: fog.explored,
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainMaterialUniform {
    color_a: Vec4,
    color_b: Vec4,
    color_c: Vec4,
    visible: u32,
    explored: u32,
}

impl From<&TerrainMaterial> for TerrainMaterialUniform {
    fn from(terrain_material: &TerrainMaterial) -> Self {
        Self {
            color_a: terrain_material.color_a.as_linear_rgba_f32().into(),
            color_b: terrain_material.color_b.as_linear_rgba_f32().into(),
            color_c: terrain_material.color_c.as_linear_rgba_f32().into(),
            visible: terrain_material.visible as u32,
            explored: terrain_material.explored as u32,
        }
    }
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/terrain_fragment.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    terrain_query: Query<(&Fog, &Handle<TerrainMaterial>), Changed<Fog>>,
) {
    for (fog, handle) in &terrain_query {
        let Some(material) = terrain_materials.get_mut(handle) else {
            continue;
        };
        material.visible = fog.visible;
        material.explored = fog.explored;
    }
}
