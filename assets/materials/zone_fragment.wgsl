#import bevy_pbr::{
    mesh_view_bindings::{view, globals},
    forward_io::VertexOutput,
    pbr_functions as fns,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
}
#import bevy_core_pipeline::tonemapping::tone_mapping
#import noisy_bevy::fbm_simplex_2d;
#import "materials/common.wgsl"::{pixel_noise, modulo};
#import "materials/zone_types.wgsl"::{
    UniformData,
    TerrainData,
    ZONE_FLAGS_VISIBLE_BIT,
    ZONE_FLAGS_EXPLORED_BIT,
    ZONE_FLAGS_HOVER_BIT
}

@group(2) @binding(0)
var<storage> terrain_data: array<TerrainData>;

@group(2) @binding(1)
var<uniform> uniform_data: UniformData;

fn cloud_noise(world_uv: vec2<f32>) -> vec4<f32> {
    let octaves = 1;
    let lacunarity = 2.0;
    let gain = 0.5;
    let cloud_uv = vec2<f32>(
        world_uv.x + cos(globals.time * 0.03),
        world_uv.y + sin(globals.time * 0.03),
    );
    let noise = fbm_simplex_2d(cloud_uv, octaves, lacunarity, gain);
    return vec4<f32>(1.0, 1.0, 1.0, (noise + 1.3) / 3.0);
}

@fragment
fn fragment(@builtin(front_facing) is_front: bool, mesh: VertexOutput) -> @location(0) vec4<f32> {
    let terrain = terrain_data[uniform_data.terrain_idx];

    var output_color = vec4<f32>(1.0);
#ifdef VERTEX_COLORS
    output_color = output_color * mesh.color
#endif
    output_color = output_color * pixel_noise(mesh.world_position.xz, terrain.color_a, terrain.color_b, terrain.color_c);

    output_color = mix(output_color, vec4<f32>(1.0), clamp(mesh.world_position.y - 0.3, 0.0, 1.0) * 2.0);

    var cloud_color = cloud_noise(mesh.world_position.xz);
    if (uniform_data.flags & ZONE_FLAGS_VISIBLE_BIT) == 0u {
        output_color = mix(output_color, cloud_color, cloud_color[3]);
    }

    if (uniform_data.flags & ZONE_FLAGS_EXPLORED_BIT) == 0u {
        output_color = pixel_noise(
            mesh.world_position.xz,
            vec4<f32>(0.005, 0.005, 0.01, 1.0),
            vec4<f32>(0.007, 0.005, 0.008, 1.0),
            vec4<f32>(0.005, 0.007, 0.008, 1.0)
        );
    }

    if (uniform_data.flags & ZONE_FLAGS_HOVER_BIT) != 0u {
        var d = length(mesh.uv - 0.5);
        var c = smoothstep(0.3, 0.5, d) * 0.7;
        output_color = mix(output_color, vec4<f32>(0.863, 0.969, 0.710, 1.0), c);
    }

    var pbr_input: PbrInput = pbr_input_new();

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.material.base_color = output_color;
    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front
    );
    pbr_input.N = normalize(pbr_input.world_normal);
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

    output_color = fns::apply_pbr_lighting(pbr_input);

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color, view.color_grading);
#endif
    return output_color;
}
