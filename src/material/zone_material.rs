use crate::{
    assets::MainAssets,
    map::Fog,
    terrain::{Height, Terrain},
};
use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::*,
};
use bevy_mod_picking::prelude::PickingInteraction;

#[derive(Default)]
pub struct ZoneMaterialPlugin;

impl Plugin for ZoneMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ZoneMaterial>::default())
            .add_systems(Update, apply_to_material);
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Clone, Default)]
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
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
    pub outer_visible: [bool; 6],
}

impl ZoneMaterial {
    pub fn new(assets: &Res<MainAssets>, terrain: &Terrain, height: &Height) -> Self {
        let terrain_texture = match terrain {
            Terrain::Ocean => Some(assets.ocean_texture.clone()),
            Terrain::Mountain => Some(assets.mountain_texture.clone()),
            Terrain::Forest => Some(assets.grass_texture.clone()),
        };

        Self {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture,
            height_amp: height.height_amp,
            height_base: height.height_base,
            outer_amp: height.outer_amp,
            outer_base: height.outer_base,
            ..default()
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct ZoneMaterialUniform {
    pub visible: u32,
    pub explored: u32,
    pub hover: u32,
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp_se: f32,
    pub outer_amp_s: f32,
    pub outer_amp_sw: f32,
    pub outer_amp_nw: f32,
    pub outer_amp_n: f32,
    pub outer_amp_ne: f32,
    pub outer_base_se: f32,
    pub outer_base_s: f32,
    pub outer_base_sw: f32,
    pub outer_base_nw: f32,
    pub outer_base_n: f32,
    pub outer_base_ne: f32,
}

impl From<&ZoneMaterial> for ZoneMaterialUniform {
    fn from(zone_material: &ZoneMaterial) -> Self {
        Self {
            visible: zone_material.visible as u32,
            explored: zone_material.explored as u32,
            hover: zone_material.hover as u32,
            height_amp: zone_material.height_amp,
            height_base: zone_material.height_base,
            outer_amp_se: zone_material.amp_for(0),
            outer_amp_s: zone_material.amp_for(1),
            outer_amp_sw: zone_material.amp_for(2),
            outer_amp_nw: zone_material.amp_for(3),
            outer_amp_n: zone_material.amp_for(4),
            outer_amp_ne: zone_material.amp_for(5),
            outer_base_se: zone_material.base_for(0),
            outer_base_s: zone_material.base_for(1),
            outer_base_sw: zone_material.base_for(2),
            outer_base_nw: zone_material.base_for(3),
            outer_base_n: zone_material.base_for(4),
            outer_base_ne: zone_material.base_for(5),
        }
    }
}

impl ZoneMaterial {
    fn amp_for(&self, idx: usize) -> f32 {
        if self.outer_visible[idx] {
            self.outer_amp[idx]
        } else {
            0.0
        }
    }

    fn base_for(&self, idx: usize) -> f32 {
        if self.outer_visible[idx] {
            self.outer_base[idx]
        } else {
            0.0
        }
    }
}

impl Material for ZoneMaterial {
    fn fragment_shader() -> ShaderRef {
        "materials/zone_fragment.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "materials/zone_vertex.wgsl".into()
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    zone_query: Query<
        (&Fog, &PickingInteraction, &Handle<ZoneMaterial>),
        Or<(Changed<Fog>, Changed<PickingInteraction>)>,
    >,
) {
    for (fog, interaction, handle) in &zone_query {
        let Some(material) = zone_materials.get_mut(handle) else { continue };
        material.visible = fog.visible;
        material.explored = fog.explored;
        material.hover = matches!(interaction, PickingInteraction::Hovered);
    }
}
