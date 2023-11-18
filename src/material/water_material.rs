use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::*,
};

#[derive(Default)]
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Clone, Default)]
#[uuid = "ce8eb070-dbb0-44c1-a1c5-d4a6ac9f4033"]
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
            color: water_material.color.as_linear_rgba_f32().into(),
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
