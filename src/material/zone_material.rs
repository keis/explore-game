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
pub struct ZoneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub terrain_texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub cloud_texture: Option<Handle<Image>>,
    #[uniform(4)]
    pub visible: u32,
    #[uniform(4)]
    pub explored: u32,
    #[uniform(4)]
    pub hover: u32,
    #[uniform(4)]
    pub height: f32,
    #[uniform(4)]
    pub outer_ne: f32,
    #[uniform(4)]
    pub outer_e: f32,
    #[uniform(4)]
    pub outer_se: f32,
    #[uniform(4)]
    pub outer_sw: f32,
    #[uniform(4)]
    pub outer_w: f32,
    #[uniform(4)]
    pub outer_nw: f32,
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
        material.visible = fog.visible as u32;
        material.explored = fog.explored as u32;
        material.hover = hover.hovered() as u32;
    }
}
