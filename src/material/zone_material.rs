use super::codex_buffer::{CodexBuffer, CodexBufferPlugin, CodexBufferValue};
use crate::terrain::{OuterVisible, Terrain, TerrainId};
use bevy::{prelude::*, render::render_resource::*};
use expl_codex::Id;
use expl_hexgrid::Neighbours;
use expl_map::Fog;

pub type TerrainBuffer = CodexBuffer<TerrainData>;

#[derive(Default)]
pub struct ZoneMaterialPlugin;

impl Plugin for ZoneMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CodexBufferPlugin::<TerrainData>::default(),
            MaterialPlugin::<ZoneMaterial>::default(),
        ))
        .add_systems(Update, apply_to_material);
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainData {
    height_base: f32,
    height_amp: f32,
    color_a: Vec4,
    color_b: Vec4,
    color_c: Vec4,
}

impl CodexBufferValue for TerrainData {
    type CodexValue = Terrain;
}

impl From<&Terrain> for TerrainData {
    fn from(value: &Terrain) -> Self {
        Self {
            height_base: value.height_base,
            height_amp: value.height_amp,
            color_a: value.color_a.to_linear().to_f32_array().into(),
            color_b: value.color_b.to_linear().to_f32_array().into(),
            color_c: value.color_c.to_linear().to_f32_array().into(),
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ZoneFlags: u32 {
        const VISIBLE = 1;
        const EXPLORED = 2;
        const OUTER_VISIBLE_E = 4;
        const OUTER_VISIBLE_SE = 8;
        const OUTER_VISIBLE_SW = 16;
        const OUTER_VISIBLE_W = 32;
        const OUTER_VISIBLE_NW = 64;
        const OUTER_VISIBLE_NE = 128;
        const HOVER = 256;
    }
}

impl From<&Fog> for ZoneFlags {
    fn from(value: &Fog) -> Self {
        let mut flags = ZoneFlags::empty();
        flags.set(ZoneFlags::VISIBLE, value.visible);
        flags.set(ZoneFlags::EXPLORED, value.explored);
        flags
    }
}

impl From<&OuterVisible> for ZoneFlags {
    fn from(value: &OuterVisible) -> Self {
        let mut flags = ZoneFlags::empty();
        flags.set(ZoneFlags::OUTER_VISIBLE_E, value[0]);
        flags.set(ZoneFlags::OUTER_VISIBLE_SE, value[1]);
        flags.set(ZoneFlags::OUTER_VISIBLE_SW, value[2]);
        flags.set(ZoneFlags::OUTER_VISIBLE_W, value[3]);
        flags.set(ZoneFlags::OUTER_VISIBLE_NW, value[4]);
        flags.set(ZoneFlags::OUTER_VISIBLE_NE, value[5]);
        flags
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct ZoneMaterialUniform {
    terrain_idx: u32,
    flags: u32,
    // Outer terrain
    outer_terrain_e: u32,
    outer_terrain_se: u32,
    outer_terrain_sw: u32,
    outer_terrain_w: u32,
    outer_terrain_nw: u32,
    outer_terrain_ne: u32,
}

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct ZoneMaterial {
    #[storage(0, read_only, buffer)]
    terrain_data: Buffer,
    #[uniform(1)]
    uniform: ZoneMaterialUniform,
}

impl ZoneMaterial {
    pub fn new(
        terrain: &TerrainId,
        fog: &Fog,
        outer_visible: &OuterVisible,
        outer_terrain: &Neighbours<Id<Terrain>>,
        terrain_buffer: &Res<TerrainBuffer>,
    ) -> Self {
        let terrain_idx = terrain_buffer.as_index(terrain).unwrap() as u32;
        let outer_terrain_idx =
            outer_terrain.map(|terrain| terrain_buffer.as_index(&terrain).unwrap() as u32);
        Self {
            terrain_data: terrain_buffer.buffer().unwrap().clone(),
            uniform: ZoneMaterialUniform {
                terrain_idx,
                flags: (ZoneFlags::from(fog) | ZoneFlags::from(outer_visible)).bits(),
                outer_terrain_e: outer_terrain_idx[0],
                outer_terrain_se: outer_terrain_idx[1],
                outer_terrain_sw: outer_terrain_idx[2],
                outer_terrain_w: outer_terrain_idx[3],
                outer_terrain_nw: outer_terrain_idx[4],
                outer_terrain_ne: outer_terrain_idx[5],
            },
        }
    }

    pub fn set_hover(&mut self, hover: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::HOVER, hover);
        self.uniform.flags = flags.bits();
    }

    pub fn set_visible(&mut self, visible: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::VISIBLE, visible);
        self.uniform.flags = flags.bits();
    }

    pub fn set_explored(&mut self, explored: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::EXPLORED, explored);
        self.uniform.flags = flags.bits();
    }

    pub fn set_outer_visible(&mut self, outer_visible: Neighbours<bool>) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::OUTER_VISIBLE_E, outer_visible[0]);
        flags.set(ZoneFlags::OUTER_VISIBLE_SE, outer_visible[1]);
        flags.set(ZoneFlags::OUTER_VISIBLE_SW, outer_visible[2]);
        flags.set(ZoneFlags::OUTER_VISIBLE_W, outer_visible[3]);
        flags.set(ZoneFlags::OUTER_VISIBLE_NW, outer_visible[4]);
        flags.set(ZoneFlags::OUTER_VISIBLE_NE, outer_visible[5]);
        self.uniform.flags = flags.bits();
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
        (&Fog, &OuterVisible, &MeshMaterial3d<ZoneMaterial>),
        Or<(Changed<Fog>, Changed<OuterVisible>)>,
    >,
) {
    for (fog, outer_visible, handle) in &zone_query {
        let Some(material) = zone_materials.get_mut(handle) else {
            continue;
        };
        material.set_visible(fog.visible);
        material.set_explored(fog.explored);
        material.set_outer_visible(**outer_visible);
    }
}

/*
fn update_from_terrain_codex(
    mut codex_events: EventReader<AssetEvent<Codex<Terrain>>>,
    terrain_codex_assets: Res<Assets<Codex<Terrain>>>,
    mut terrain_buffer: ResMut<TerrainBuffer>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for event in codex_events.read() {
        let AssetEvent::Modified { id: asset_id } = event else {
            continue;
        };
        let terrain_codex = terrain_codex_assets.get(*asset_id).unwrap();
        terrain_buffer.extend(terrain_codex.iter());
    }
    terrain_buffer
        .data
        .write_buffer(&render_device, &render_queue);
}
*/
