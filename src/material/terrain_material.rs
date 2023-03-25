use crate::map::Fog;
use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::*};

#[derive(Default)]
pub struct TerrainMaterialPlugin;

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TerrainMaterial>::default())
            .add_system(apply_to_material);
    }
}

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "f3c06773-d878-40b4-8f00-f39b82513c81"]
#[uniform(0, TerrainMaterialUniform)]
pub struct TerrainMaterial {
    pub color: Color,
    pub visible: bool,
    pub explored: bool,
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainMaterialUniform {
    pub color: Vec4,
    pub visible: u32,
    pub explored: u32,
}

impl From<&TerrainMaterial> for TerrainMaterialUniform {
    fn from(terrain_material: &TerrainMaterial) -> Self {
        Self {
            color: terrain_material.color.as_linear_rgba_f32().into(),
            visible: terrain_material.visible as u32,
            explored: terrain_material.explored as u32,
        }
    }
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    terrain_query: Query<(&Fog, &Handle<TerrainMaterial>), Changed<Fog>>,
) {
    for (fog, handle) in &terrain_query {
        let Some(mut material) = terrain_materials.get_mut(handle) else { continue };
        material.visible = fog.visible;
        material.explored = fog.explored;
    }
}
