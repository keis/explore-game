#import bevy_pbr::{
  mesh_functions,
  forward_io::{Vertex, VertexOutput},
  view_transformations::position_world_to_clip,
}
#import noisy_bevy::simplex_noise_2d
#import "materials/zone_types.wgsl"::{
    UniformData,
    TerrainData,
    ZONE_FLAGS_EXPLORED_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_E_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_SE_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_SW_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_W_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_NW_BIT,
    ZONE_FLAGS_OUTER_VISIBLE_NE_BIT
}

@group(2) @binding(0)
var<storage> terrain_data: array<TerrainData>;

@group(2) @binding(1)
var<uniform> uniform_data: UniformData;

fn corner(self_value: f32, a_value: f32, b_value: f32) -> f32 {
    var min_value = min(self_value, min(a_value, b_value));
    var max_value = max(self_value, max(a_value, b_value));
    var result: f32;

    if min_value < 0.0 && max_value < 0.0 {
        result = max_value;
    } else if min_value < 0.0 {
        result = 0.0;
    } else {
        result = min_value;
    }

    return result;
}

fn edge(self_value: f32, a_value: f32) -> f32 {
    var min_value = min(self_value, a_value);
    var max_value = max(self_value, a_value);
    var result: f32;

    if min_value < 0.0 && max_value < 0.0 {
        result = max_value;
    } else if min_value < 0.0 {
        result = 0.0;
    } else {
        result = min_value;
    }

    return result;
}

fn amp_and_base(position: vec2<f32>) -> vec2<f32> {
    let terrain = terrain_data[uniform_data.terrain_idx];

    let outer_amp_e = select(0.0, terrain_data[uniform_data.outer_terrain_e].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_E_BIT) != 0u);
    let outer_amp_se = select(0.0, terrain_data[uniform_data.outer_terrain_se].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_SE_BIT) != 0u);
    let outer_amp_sw = select(0.0, terrain_data[uniform_data.outer_terrain_sw].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_SW_BIT) != 0u);
    let outer_amp_w = select(0.0, terrain_data[uniform_data.outer_terrain_w].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_W_BIT) != 0u);
    let outer_amp_nw = select(0.0, terrain_data[uniform_data.outer_terrain_nw].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_NW_BIT) != 0u);
    let outer_amp_ne = select(0.0, terrain_data[uniform_data.outer_terrain_ne].height_amp, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_NE_BIT) != 0u);

    let outer_base_e = select(0.0, terrain_data[uniform_data.outer_terrain_e].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_E_BIT) != 0u);
    let outer_base_se = select(0.0, terrain_data[uniform_data.outer_terrain_se].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_SE_BIT) != 0u);
    let outer_base_sw = select(0.0, terrain_data[uniform_data.outer_terrain_sw].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_SW_BIT) != 0u);
    let outer_base_w = select(0.0, terrain_data[uniform_data.outer_terrain_w].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_W_BIT) != 0u);
    let outer_base_nw = select(0.0, terrain_data[uniform_data.outer_terrain_nw].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_NW_BIT) != 0u);
    let outer_base_ne = select(0.0, terrain_data[uniform_data.outer_terrain_ne].height_base, (uniform_data.flags & ZONE_FLAGS_OUTER_VISIBLE_NE_BIT) != 0u);

    // South corner
    if position.y >= 0.9 {
        return vec2<f32>(
            corner(terrain.height_amp, outer_amp_se, outer_amp_sw),
            corner(terrain.height_base, outer_base_se, outer_base_sw),
        );
    }

    // North Corner
    if position.y <= -0.9 {
        return vec2<f32>(
            corner(terrain.height_amp, outer_amp_nw, outer_amp_ne),
            corner(terrain.height_base, outer_base_nw, outer_base_ne),
        );
    }

    if position.x > 0.0 {
        // South-East Corner or South-East Edge
        if position.y > position.x * -0.5 + 0.9 {
            if position.x > 0.8 {
                return vec2<f32>(
                    corner(terrain.height_amp, outer_amp_e, outer_amp_se),
                    corner(terrain.height_base, outer_base_e, outer_base_se),
                );
            }
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_se),
                edge(terrain.height_base, outer_base_se),
            );
        }
        // North-East Corner or North-East Edge
        if position.y < position.x * 0.5 - 0.9 {
            if position.x > 0.8 {
                return vec2<f32>(
                    corner(terrain.height_amp, outer_amp_ne, outer_amp_e),
                    corner(terrain.height_base, outer_base_ne, outer_base_e),
                );
            }
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_ne),
                edge(terrain.height_base, outer_base_ne),
            );
        }
        // East Edge
        if position.x > 0.8 {
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_e),
                edge(terrain.height_base, outer_base_e),
            );
        }
    }

    if position.x < 0.0 {
        // South-West Corner or South-West Edge
        if position.y > -position.x * -0.5 + 0.9 {
            if position.x < -0.8 {
                return vec2<f32>(
                    corner(terrain.height_amp, outer_amp_w, outer_amp_sw),
                    corner(terrain.height_base, outer_base_w, outer_base_sw),
                );
            }
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_sw),
                edge(terrain.height_base, outer_base_sw),
            );
        }
        // North-West Corner or North-West Edge
        if position.y < -position.x * 0.5 - 0.9 {
            if position.x < -0.8 {
                return vec2<f32>(
                    corner(terrain.height_amp, outer_amp_nw, outer_amp_w),
                    corner(terrain.height_base, outer_base_nw, outer_base_w),
                );
            }
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_nw),
                edge(terrain.height_base, outer_base_nw),
            );
        }
        // West Edge
        if position.x < -0.8 {
            return vec2<f32>(
                edge(terrain.height_amp, outer_amp_w),
                edge(terrain.height_base, outer_base_w),
            );
        }
    }

    // Internal
    if (uniform_data.flags & ZONE_FLAGS_EXPLORED_BIT) != 0u {
        return vec2<f32>(terrain.height_amp, terrain.height_base);
    }
    return vec2<f32>(0.0, 0.0);
}

fn height_at(position: vec2<f32>, world_position: vec2<f32>) -> f32 {
    var k = amp_and_base(position);
    var noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
    return k.y + noise * k.x;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef SKINNED
    var model = bevy_pbr::skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var model = mesh_functions::get_world_from_local(vertex.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(model, vertex.tangent);
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    let worldxz = out.world_position.xz;
    let offset = vec3<f32>(-0.001, 0.0, 0.001);
    out.world_position.y = height_at(vertex.position.xz, worldxz);
    let a = height_at(vertex.position.xz + offset.xy, worldxz + offset.xy);
    let b = height_at(vertex.position.xz + offset.zy, worldxz + offset.zy);
    let c = height_at(vertex.position.xz + offset.yx, worldxz + offset.yx);
    let d = height_at(vertex.position.xz + offset.yz, worldxz + offset.yz);

    out.world_normal = normalize(vec3<f32>((a - b) / 2.0, 1.0, (c - d) / 2.0));
    out.position = position_world_to_clip(out.world_position.xyz);

    return out;
}
