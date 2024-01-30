const ZONE_FLAGS_VISIBLE_BIT: u32 = 1u;
const ZONE_FLAGS_EXPLORED_BIT: u32 = 2u;
const ZONE_FLAGS_OUTER_VISIBLE_E_BIT: u32 = 4u;
const ZONE_FLAGS_OUTER_VISIBLE_SE_BIT: u32 = 8u;
const ZONE_FLAGS_OUTER_VISIBLE_SW_BIT: u32 = 16u;
const ZONE_FLAGS_OUTER_VISIBLE_W_BIT: u32 = 32u;
const ZONE_FLAGS_OUTER_VISIBLE_NW_BIT: u32 = 64u;
const ZONE_FLAGS_OUTER_VISIBLE_NE_BIT: u32 = 128u;
const ZONE_FLAGS_HOVER_BIT: u32 = 256u;

struct UniformData {
    terrain_idx: u32,
    flags: u32,
    outer_terrain_e: u32,
    outer_terrain_se: u32,
    outer_terrain_sw: u32,
    outer_terrain_w: u32,
    outer_terrain_nw: u32,
    outer_terrain_ne: u32,
}

struct TerrainData {
    height_base: f32,
    height_amp: f32,
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    color_c: vec4<f32>,
}
