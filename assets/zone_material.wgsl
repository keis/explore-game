struct UniformData {
    visible: u32,
    explored: u32,
    time: f32,
}

@group(1) @binding(0)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(1)
var terrain_texture_sampler: sampler;
@group(1) @binding(2)
var cloud_texture: texture_2d<f32>;
@group(1) @binding(3)
var cloud_texture_sampler: sampler;
@group(1) @binding(4)
var<uniform> uniform_data: UniformData;

fn modulo(a: f32, n: f32) -> f32 {
    return a - n * floor(a / n);
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var uv = vec2<f32>(
        modulo(world_position.x * 0.2, 1.0),
        modulo(world_position.z * 0.2, 1.0)
    );
    var output_color = textureSample(terrain_texture, terrain_texture_sampler, uv);
    if uniform_data.visible == 0u {
        var cloud_uv = vec2<f32>(
            modulo(uv.x + cos(uniform_data.time * 0.01), 1.0),
            modulo(uv.y + sin(uniform_data.time * 0.01), 1.0)
        );
        var cloud_color = textureSample(cloud_texture, cloud_texture_sampler, cloud_uv);
        output_color = vec4<f32>(
            mix(output_color[0], cloud_color[0], cloud_color[3] * 0.7),
            mix(output_color[1], cloud_color[1], cloud_color[3] * 0.7),
            mix(output_color[2], cloud_color[2], cloud_color[3] * 0.7),
            1.0
        );

    }
    if uniform_data.explored == 0u {
        output_color = vec4<f32>(0.005, 0.005, 0.01, 1.0);
    }
    return output_color;
}
