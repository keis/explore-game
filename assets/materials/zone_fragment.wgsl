#import bevy_pbr::mesh_view_bindings view, globals
#import bevy_pbr::pbr_functions as fns
#import bevy_pbr::pbr_types STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT
#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_core_pipeline::tonemapping tone_mapping

struct UniformData {
    visible: u32,
    explored: u32,
    hover: u32,
    height: f32,
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
fn fragment(@builtin(front_facing) is_front: bool, mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
    var output_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
#ifdef VERTEX_COLORS
    output_color = output_color * mesh.color
#endif
    var world_uv = vec2<f32>(
        modulo(mesh.world_position.x * 0.3, 1.0),
        modulo(mesh.world_position.z * 0.3, 1.0)
    );
    output_color = output_color * textureSample(terrain_texture, terrain_texture_sampler, world_uv);
    output_color = mix(output_color, vec4<f32>(1.0, 1.0, 1.0, 1.0), clamp(mesh.world_position.y - 0.3, 0.0, 1.0) * 2.0);
    var cloud_uv = vec2<f32>(
        modulo(world_uv.x + cos(globals.time * 0.01), 1.0),
        modulo(world_uv.y + sin(globals.time * 0.01), 1.0)
    );
    var cloud_color = textureSample(cloud_texture, cloud_texture_sampler, cloud_uv);
    if uniform_data.visible == 0u {
        output_color = mix(output_color, vec4<f32>(cloud_color.xyz, 1.0), cloud_color[3] * 0.7);
    }

    if uniform_data.explored == 0u {
        output_color = vec4<f32>(0.005, 0.005, 0.01, 1.0);
    }

    if uniform_data.hover == 1u {
        var d = length(mesh.uv - 0.5);
        var c = smoothstep(0.3, 0.5, d) * 0.7;
        output_color = mix(output_color, vec4<f32>(0.863, 0.969, 0.710, 1.0), c);
    }

    var pbr_input: fns::PbrInput = fns::pbr_input_new();

    pbr_input.material.base_color = output_color;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        is_front
    );
    pbr_input.N = fns::apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        #ifdef VERTEX_TANGENTS
        #ifdef STANDARDMATERIAL_NORMAL_MAP
        mesh.world_tangent,
        #endif
        #endif
        world_uv,
        view.mip_bias,
    );
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);
    //pbr_input.flags = mesh.flags;

    output_color = fns::pbr(pbr_input);

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color, view.color_grading);
#endif
    return output_color;
}
