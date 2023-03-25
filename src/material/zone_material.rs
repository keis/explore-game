use crate::map::Fog;
use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::*};
use bevy_mod_picking::Hover;

#[derive(Default)]
pub struct ZoneMaterialPlugin;

impl Plugin for ZoneMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<ZoneMaterial>::default())
            .add_system(apply_to_material);
    }
}

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "05f50382-7218-4860-8c4c-06dbd66694db"]
#[uniform(4, ZoneMaterialUniform)]
pub struct ZoneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub terrain_texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub cloud_texture: Option<Handle<Image>>,
    pub visible: bool,
    pub explored: bool,
    pub hover: bool,
    pub height: f32,
    pub outer_ne: f32,
    pub outer_e: f32,
    pub outer_se: f32,
    pub outer_sw: f32,
    pub outer_w: f32,
    pub outer_nw: f32,
}

#[derive(Clone, Default, ShaderType)]
pub struct ZoneMaterialUniform {
    pub visible: u32,
    pub explored: u32,
    pub hover: u32,
    pub height: f32,
    pub outer_ne: f32,
    pub outer_e: f32,
    pub outer_se: f32,
    pub outer_sw: f32,
    pub outer_w: f32,
    pub outer_nw: f32,
}

impl From<&ZoneMaterial> for ZoneMaterialUniform {
    fn from(zone_material: &ZoneMaterial) -> Self {
        Self {
            visible: zone_material.visible as u32,
            explored: zone_material.explored as u32,
            hover: zone_material.hover as u32,
            height: zone_material.height,
            outer_ne: zone_material.outer_ne,
            outer_e: zone_material.outer_e,
            outer_se: zone_material.outer_se,
            outer_sw: zone_material.outer_sw,
            outer_w: zone_material.outer_w,
            outer_nw: zone_material.outer_nw,
        }
    }
}

impl Material for ZoneMaterial {
    fn fragment_shader() -> ShaderRef {
        "zone_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "zone_material_vertex.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    zone_query: Query<(&Fog, &Hover, &Handle<ZoneMaterial>), Or<(Changed<Fog>, Changed<Hover>)>>,
) {
    for (fog, hover, handle) in &zone_query {
        let Some(mut material) = zone_materials.get_mut(handle) else { continue };
        material.visible = fog.visible;
        material.explored = fog.explored;
        material.hover = hover.hovered();
    }
}
