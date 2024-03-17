use bevy::{prelude::*, reflect::TypePath, render::render_resource::*};

#[derive(Default)]
pub struct PortalMaterialPlugin;

impl Plugin for PortalMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PortalMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
#[uniform(0, PortalMaterialUniform)]
pub struct PortalMaterial {
    pub base_color: Color,
    pub swirl_color: Color,
}

#[derive(Clone, Default, ShaderType)]
pub struct PortalMaterialUniform {
    pub base_color: Vec4,
    pub swirl_color: Vec4,
}

impl From<&PortalMaterial> for PortalMaterialUniform {
    fn from(portal_material: &PortalMaterial) -> Self {
        Self {
            base_color: portal_material.base_color.as_linear_rgba_f32().into(),
            swirl_color: portal_material.swirl_color.as_linear_rgba_f32().into(),
        }
    }
}

impl Material for PortalMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/portal_fragment.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
