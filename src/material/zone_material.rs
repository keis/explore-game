use crate::{
    assets::MainAssets,
    map::Fog,
    terrain::{Codex, Height, OuterVisible, Terrain, TerrainId},
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

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Clone, Default)]
#[uuid = "05f50382-7218-4860-8c4c-06dbd66694db"]
#[uniform(4, ZoneMaterialUniform)]
pub struct ZoneMaterial {
    #[texture(2)]
    #[sampler(3)]
    pub cloud_texture: Option<Handle<Image>>,
    pub visible: bool,
    pub explored: bool,
    pub hover: bool,
    pub color_a: Color,
    pub color_b: Color,
    pub color_c: Color,
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
    pub outer_visible: [bool; 6],
}

impl ZoneMaterial {
    pub fn new(
        assets: &Res<MainAssets>,
        terrain_codex: &Codex<Terrain>,
        terrain: &TerrainId,
        height: &Height,
        fog: &Fog,
        outer_visible: &OuterVisible,
    ) -> Self {
        let terrain_data = &terrain_codex[terrain];

        Self {
            cloud_texture: Some(assets.cloud_texture.clone()),
            color_a: terrain_data.color_a,
            color_b: terrain_data.color_b,
            color_c: terrain_data.color_c,
            visible: fog.visible,
            explored: fog.explored,
            height_amp: height.height_amp,
            height_base: height.height_base,
            outer_amp: height.outer_amp.into(),
            outer_base: height.outer_base.into(),
            outer_visible: **outer_visible,
            ..default()
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct ZoneMaterialUniform {
    pub visible: u32,
    pub explored: u32,
    pub hover: u32,
    pub color_a: Vec4,
    pub color_b: Vec4,
    pub color_c: Vec4,
    pub height_amp: f32,
    pub height_base: f32,
    // Outer amplifier
    pub outer_amp_e: f32,
    pub outer_amp_se: f32,
    pub outer_amp_sw: f32,
    pub outer_amp_w: f32,
    pub outer_amp_nw: f32,
    pub outer_amp_ne: f32,
    // Outer base
    pub outer_base_e: f32,
    pub outer_base_se: f32,
    pub outer_base_sw: f32,
    pub outer_base_w: f32,
    pub outer_base_nw: f32,
    pub outer_base_ne: f32,
}

impl From<&ZoneMaterial> for ZoneMaterialUniform {
    fn from(zone_material: &ZoneMaterial) -> Self {
        Self {
            visible: zone_material.visible as u32,
            explored: zone_material.explored as u32,
            hover: zone_material.hover as u32,
            color_a: zone_material.color_a.as_linear_rgba_f32().into(),
            color_b: zone_material.color_b.as_linear_rgba_f32().into(),
            color_c: zone_material.color_c.as_linear_rgba_f32().into(),
            height_amp: zone_material.height_amp,
            height_base: zone_material.height_base,
            outer_amp_e: zone_material.amp_for(0),
            outer_amp_se: zone_material.amp_for(1),
            outer_amp_sw: zone_material.amp_for(2),
            outer_amp_w: zone_material.amp_for(3),
            outer_amp_nw: zone_material.amp_for(4),
            outer_amp_ne: zone_material.amp_for(5),
            outer_base_e: zone_material.base_for(0),
            outer_base_se: zone_material.base_for(1),
            outer_base_sw: zone_material.base_for(2),
            outer_base_w: zone_material.base_for(3),
            outer_base_nw: zone_material.base_for(4),
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
        (
            &Fog,
            &PickingInteraction,
            &OuterVisible,
            &Handle<ZoneMaterial>,
        ),
        Or<(
            Changed<Fog>,
            Changed<PickingInteraction>,
            Changed<OuterVisible>,
        )>,
    >,
) {
    for (fog, interaction, outer_visible, handle) in &zone_query {
        let Some(material) = zone_materials.get_mut(handle) else {
            continue;
        };
        material.visible = fog.visible;
        material.explored = fog.explored;
        material.hover = matches!(interaction, PickingInteraction::Hovered);
        material.outer_visible = **outer_visible;
    }
}
