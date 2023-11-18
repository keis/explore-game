#import bevy_pbr::{
    mesh_view_bindings::{view, globals},
    forward_io::VertexOutput
}

struct UniformData {
    base_color: vec4<f32>,
    swirl_color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> uniform_data: UniformData;

fn modulo(a: f32, n: f32) -> f32 {
    return a - n * floor(a / n);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var uv = floor(mesh.uv * 16.0) / 16.0;

    var p = -1.0 + 2.0 * uv;
    var w = sin(globals.time + 6.5 * sqrt(dot(p, p)) * cos(p.x));
    var x = cos(6.0 * atan(p.y / p.x) + 1.8 * w);

    var output_color = mix(uniform_data.base_color, uniform_data.swirl_color, x);
    return output_color * (1.0 - smoothstep(0.20, 0.45, length(mesh.uv - 0.5)));
}
