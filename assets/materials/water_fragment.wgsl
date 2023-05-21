#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::pbr_ambient
#import bevy_pbr::shadows
#import bevy_pbr::fog
#import bevy_pbr::pbr_functions

#import noisy_bevy::prelude

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

struct UniformData {
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> uniform_data: UniformData;

fn modulo(a: f32, n: f32) -> f32 {
    return a - n * floor(a / n);
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var output_color = uniform_data.color;
#ifdef VERTEX_COLORS
    output_color = output_color * in.color
#endif
    var world_uv = floor(in.world_position.xz * 16.0) / 16.0;

    var octaves = 2;
    var lacunarity = 2.0;
    var gain = 0.5;
    var limit = 0.07;
    var border = 0.08;
    var speed = 0.02;
    var noise = fbm_simplex_2d(world_uv - globals.time * speed, octaves, lacunarity, gain) * fbm_simplex_2d(world_uv + globals.time * speed, octaves, lacunarity, gain);
    var e = smoothstep(limit - border, limit, noise) - smoothstep(limit, limit + border, noise);
    output_color = mix(output_color, vec4<f32>(e * e, e * e, e * e, 1.0), 0.4);
    return output_color;
}
