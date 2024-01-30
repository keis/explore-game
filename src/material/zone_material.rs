use crate::{
    assets::{AssetState, CodexAssets},
    input::{ZoneOut, ZoneOver},
    map::Fog,
    terrain::{OuterVisible, Terrain, TerrainCodex, TerrainId},
};
use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
    },
};
use expl_codex::{Codex, Id};
use expl_hexgrid::Neighbours;

#[derive(Default)]
pub struct ZoneMaterialPlugin;

impl Plugin for ZoneMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ZoneMaterial>::default())
            .init_resource::<TerrainBuffer>()
            .add_systems(
                Update,
                (
                    update_from_terrain_codex
                        .run_if(on_event::<AssetEvent<Codex<Terrain>>>())
                        .run_if(in_state(AssetState::Loaded)),
                    update_hover.run_if(on_event::<ZoneOver>().or_else(on_event::<ZoneOut>())),
                    apply_to_material,
                ),
            )
            .add_systems(OnEnter(AssetState::Loaded), prepare_terrain_buffer);
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainData {
    pub height_base: f32,
    pub height_amp: f32,
    pub color_a: Vec4,
    pub color_b: Vec4,
    pub color_c: Vec4,
}

impl From<&Terrain> for TerrainData {
    fn from(value: &Terrain) -> Self {
        Self {
            height_base: value.height_base,
            height_amp: value.height_amp,
            color_a: value.color_a.as_linear_rgba_f32().into(),
            color_b: value.color_b.as_linear_rgba_f32().into(),
            color_c: value.color_c.as_linear_rgba_f32().into(),
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

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Clone)]
#[uuid = "05f50382-7218-4860-8c4c-06dbd66694db"]
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
            terrain_data: terrain_buffer.data.buffer().unwrap().clone(),
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

    fn set_hover(&mut self, hover: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::HOVER, hover);
        self.uniform.flags = flags.bits();
    }

    fn set_visible(&mut self, visible: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::VISIBLE, visible);
        self.uniform.flags = flags.bits();
    }

    fn set_explored(&mut self, explored: bool) {
        let mut flags = ZoneFlags::from_bits_retain(self.uniform.flags);
        flags.set(ZoneFlags::EXPLORED, explored);
        self.uniform.flags = flags.bits();
    }

    fn set_outer_visible(&mut self, outer_visible: Neighbours<bool>) {
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

#[derive(Resource, Default)]
pub struct TerrainBuffer {
    keys: Vec<Id<Terrain>>,
    data: StorageBuffer<Vec<TerrainData>>,
}

impl TerrainBuffer {
    fn as_index(&self, terrain_id: &Id<Terrain>) -> Option<usize> {
        self.keys.iter().position(|&key| key == *terrain_id)
    }
}

impl<'a> Extend<(&'a Id<Terrain>, &'a Terrain)> for TerrainBuffer {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (&'a Id<Terrain>, &'a Terrain)>,
    {
        for (terrain_id, terrain) in iter {
            if let Some(position) = self.as_index(terrain_id) {
                self.data.get_mut()[position] = terrain.into();
            } else {
                self.keys.push(*terrain_id);
                self.data.get_mut().push(terrain.into());
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_to_material(
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    zone_query: Query<
        (&Fog, &OuterVisible, &Handle<ZoneMaterial>),
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

fn update_hover(
    mut out_events: EventReader<ZoneOut>,
    mut over_events: EventReader<ZoneOver>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    material_query: Query<&Handle<ZoneMaterial>>,
) {
    for ZoneOut(entity) in out_events.read() {
        let Ok(handle) = material_query.get(*entity) else {
            continue;
        };
        let Some(material) = zone_materials.get_mut(handle) else {
            continue;
        };
        material.set_hover(false);
    }
    for ZoneOver(entity) in over_events.read() {
        let Ok(handle) = material_query.get(*entity) else {
            continue;
        };
        let Some(material) = zone_materials.get_mut(handle) else {
            continue;
        };
        material.set_hover(true);
    }
}

fn prepare_terrain_buffer(
    mut terrain_buffer: ResMut<TerrainBuffer>,
    terrain_codex: TerrainCodex,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Ok(terrain_codex) = terrain_codex.get() else {
        return;
    };
    terrain_buffer.extend(terrain_codex.iter());
    terrain_buffer
        .data
        .write_buffer(&render_device, &render_queue);
}

fn update_from_terrain_codex(
    codex_assets: Res<CodexAssets>,
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
        if *asset_id == codex_assets.terrain_codex.id() {
            let terrain_codex = terrain_codex_assets
                .get(codex_assets.terrain_codex.clone())
                .unwrap();
            terrain_buffer.extend(terrain_codex.iter());
            terrain_buffer
                .data
                .write_buffer(&render_device, &render_queue);
        }
    }
}
