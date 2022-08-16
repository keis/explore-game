struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
};

struct UniformData {
    visible: u32,
    explored: u32,
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var<uniform> uniform_data: UniformData;

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    var output_color = textureSample(texture, texture_sampler, input.uv);
    if uniform_data.visible == 0u {
        output_color = output_color * vec4<f32>(0.2, 0.2, 0.2, 1.0);
    }
    if uniform_data.explored == 0u {
        output_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return output_color;
}
