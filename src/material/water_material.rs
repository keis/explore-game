use bevy::{prelude::*, reflect::TypePath, render::render_resource::*};

#[derive(Default)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
#[uniform(0, WaterMaterialUniform)]
pub struct WaterMaterial {
    pub color: Color,
}

#[derive(Clone, Default, ShaderType)]
pub struct WaterMaterialUniform {
    pub color: Vec4,
}

impl From<&WaterMaterial> for WaterMaterialUniform {
    fn from(water_material: &WaterMaterial) -> Self {
        Self {
            color: water_material.color.to_linear().to_f32_array().into(),
        }
    }
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/water_fragment.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
