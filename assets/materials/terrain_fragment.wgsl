#import bevy_pbr::{
  mesh_view_bindings::{view, globals},
  forward_io::VertexOutput,
  pbr_functions as fns,
  pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
}
#import bevy_core_pipeline::tonemapping::tone_mapping
#import "materials/common.wgsl"::modulo;

struct UniformData {
    color: vec4<f32>,
    visible: u32,
    explored: u32,
}

@group(1) @binding(0)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(1)
var terrain_texture_sampler: sampler;
@group(1) @binding(2)
var<uniform> uniform_data: UniformData;

@fragment
fn fragment(@builtin(front_facing) is_front: bool, mesh: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = uniform_data.color;
#ifdef VERTEX_COLORS
    output_color = output_color * mesh.color
#endif
    var world_uv = vec2<f32>(
        modulo(mesh.world_position.x * 0.5 + floor(mesh.world_position.y * 5.0) * 0.1, 1.0),
        modulo(mesh.world_position.z * 0.5 + floor(mesh.world_position.y * 5.0) * 0.1, 1.0)
    );
    var texture_color = textureSample(terrain_texture, terrain_texture_sampler, world_uv);
    output_color *= texture_color;

    if uniform_data.visible == 0u && uniform_data.explored == 1u {
        output_color = mix(output_color, vec4<f32>(0.2, 0.2, 0.2, 1.0), 0.7);
    }

    var pbr_input: PbrInput = pbr_input_new();

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.material.base_color = output_color;
    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front
    );
    pbr_input.N = fns::apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        double_sided,
        is_front,
        #ifdef VERTEX_TANGENTS
        #ifdef STANDARDMATERIAL_NORMAL_MAP
        mesh.world_tangent,
        #endif
        #endif
        mesh.uv,
        view.mip_bias,
    );
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);
    //pbr_input.flags = mesh.flags;

    output_color = fns::apply_pbr_lighting(pbr_input);

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color, view.color_grading);
#endif
    return output_color;
}
