#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput
}
#import noisy_bevy::fbm_simplex_2d
#import "materials/common.wgsl"::modulo;

struct UniformData {
    color: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> uniform_data: UniformData;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = uniform_data.color;
#ifdef VERTEX_COLORS
    output_color = output_color * mesh.color
#endif
    var world_uv = floor(mesh.world_position.xz * 16.0) / 16.0;

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
