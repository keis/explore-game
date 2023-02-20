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

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "f3c06773-d878-40b4-8f00-f39b82513c81"]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub visible: u32,
    #[uniform(0)]
    pub explored: u32,
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
        material.visible = fog.visible as u32;
        material.explored = fog.explored as u32;
    }
}
