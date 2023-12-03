#import bevy_pbr::{
  mesh_functions,
  forward_io::{Vertex, VertexOutput},
  view_transformations::position_world_to_clip,
}
#import bevy_render::instance_index::get_instance_index
#import noisy_bevy::simplex_noise_2d

struct UniformData {
    visible: u32,
    explored: u32,
    hover: u32,
    color_a: vec4<f32>,
    color_b: vec4<f32>,
    color_c: vec4<f32>,
    height_amp: f32,
    height_base: f32,
    // Outer amplifier
    outer_amp_e: f32,
    outer_amp_se: f32,
    outer_amp_sw: f32,
    outer_amp_w: f32,
    outer_amp_nw: f32,
    outer_amp_ne: f32,
    // Outer base
    outer_base_e: f32,
    outer_base_se: f32,
    outer_base_sw: f32,
    outer_base_w: f32,
    outer_base_nw: f32,
    outer_base_ne: f32,
}

@group(1) @binding(4)
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
    // South corner
    if position.y >= 0.9 {
        return vec2<f32>(
            corner(uniform_data.height_amp, uniform_data.outer_amp_se, uniform_data.outer_amp_sw),
            corner(uniform_data.height_base, uniform_data.outer_base_se, uniform_data.outer_base_sw),
        );
    }

    // North Corner
    if position.y <= -0.9 {
        return vec2<f32>(
            corner(uniform_data.height_amp, uniform_data.outer_amp_nw, uniform_data.outer_amp_ne),
            corner(uniform_data.height_base, uniform_data.outer_base_nw, uniform_data.outer_base_ne),
        );
    }

    if position.x > 0.0 {
        // South-East Corner or South-East Edge
        if position.y > position.x * -0.5 + 0.9 {
            if position.x > 0.8 {
                return vec2<f32>(
                    corner(uniform_data.height_amp, uniform_data.outer_amp_e, uniform_data.outer_amp_se),
                    corner(uniform_data.height_base, uniform_data.outer_base_e, uniform_data.outer_base_se),
                );
            }
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_se),
                edge(uniform_data.height_base, uniform_data.outer_base_se),
            );
        }
        // North-East Corner or North-East Edge
        if position.y < position.x * 0.5 - 0.9 {
            if position.x > 0.8 {
                return vec2<f32>(
                    corner(uniform_data.height_amp, uniform_data.outer_amp_ne, uniform_data.outer_amp_e),
                    corner(uniform_data.height_base, uniform_data.outer_base_ne, uniform_data.outer_base_e),
                );
            }
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_ne),
                edge(uniform_data.height_base, uniform_data.outer_base_ne),
            );
        }
        // East Edge
        if position.x > 0.8 {
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_e),
                edge(uniform_data.height_base, uniform_data.outer_base_e),
            );
        }
    }

    if position.x < 0.0 {
        // South-West Corner or South-West Edge
        if position.y > -position.x * -0.5 + 0.9 {
            if position.x < -0.8 {
                return vec2<f32>(
                    corner(uniform_data.height_amp, uniform_data.outer_amp_w, uniform_data.outer_amp_sw),
                    corner(uniform_data.height_base, uniform_data.outer_base_w, uniform_data.outer_base_sw),
                );
            }
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_sw),
                edge(uniform_data.height_base, uniform_data.outer_base_sw),
            );
        }
        // North-West Corner or North-West Edge
        if position.y < -position.x * 0.5 - 0.9 {
            if position.x < -0.8 {
                return vec2<f32>(
                    corner(uniform_data.height_amp, uniform_data.outer_amp_nw, uniform_data.outer_amp_w),
                    corner(uniform_data.height_base, uniform_data.outer_base_nw, uniform_data.outer_base_w),
                );
            }
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_nw),
                edge(uniform_data.height_base, uniform_data.outer_base_nw),
            );
        }
        // West Edge
        if position.x < -0.8 {
            return vec2<f32>(
                edge(uniform_data.height_amp, uniform_data.outer_amp_w),
                edge(uniform_data.height_base, uniform_data.outer_base_w),
            );
        }
    }

    // Internal
    if uniform_data.explored == 1u {
        return vec2<f32>(uniform_data.height_amp, uniform_data.height_base);
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
    var model = mesh_functions::get_model_matrix(vertex.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        get_instance_index(vertex.instance_index)
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
