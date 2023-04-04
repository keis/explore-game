#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import noisy_bevy::prelude

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct UniformData {
    visible: u32,
    explored: u32,
    hover: u32,
    height_amp: f32,
    height_base: f32,
    outer_amp_se: f32,
    outer_amp_s: f32,
    outer_amp_sw: f32,
    outer_amp_nw: f32,
    outer_amp_n: f32,
    outer_amp_ne: f32,
    outer_base_se: f32,
    outer_base_s: f32,
    outer_base_sw: f32,
    outer_base_nw: f32,
    outer_base_n: f32,
    outer_base_ne: f32,
}

@group(1) @binding(4)
var<uniform> uniform_data: UniformData;

struct Vertex {
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

fn height_at(position: vec2<f32>, world_position: vec2<f32>) -> f32 {
    var amp: f32;
    var base: f32;
    var dc = length(position);
    var da = length(abs(position) - vec2<f32>(0.8660254, 0.5));
    var db = length(abs(position) - vec2<f32>(0.0, 1.0));
    if dc < 0.7 {
        if uniform_data.explored == 1u {
            amp = uniform_data.height_amp;
            base = uniform_data.height_base;
        } else {
            amp = 0.0;
            base = 0.0;
        }
    } else if da < db {
        if position.x > 0.0 {
            if position.y > 0.0 {
                amp = uniform_data.outer_amp_se;
                base = uniform_data.outer_base_se;
            } else {
                amp = uniform_data.outer_amp_ne;
                base = uniform_data.outer_base_ne;
            }
        } else {
            if position.y > 0.0 {
                amp = uniform_data.outer_amp_sw;
                base = uniform_data.outer_base_sw;
            } else {
                amp = uniform_data.outer_amp_nw;
                base = uniform_data.outer_base_nw;
            }
        }
    } else {
        if position.y > 0.0 {
            amp = uniform_data.outer_amp_s;
            base = uniform_data.outer_base_s;
        } else {
            amp = uniform_data.outer_amp_n;
            base = uniform_data.outer_base_n;
        }
    }

    var noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
    return base + noise * amp;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var model = mesh.model;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh_position_world_to_clip(out.world_position);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
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
    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}
