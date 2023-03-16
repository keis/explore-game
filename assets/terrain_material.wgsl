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

struct UniformData {
    color: vec4<f32>,
    visible: u32,
    explored: u32,
}

@group(1) @binding(0)
var<uniform> uniform_data: UniformData;

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var base_color = uniform_data.color;

    if uniform_data.visible == 0u && uniform_data.explored == 1u {
        base_color = vec4<f32>(
            0.2,
            0.2,
            0.2,
            1.0
        );
    }

    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.base_color = base_color;
    pbr_input.frag_coord = position;
    pbr_input.world_position = world_position;
    pbr_input.world_normal = world_normal;
    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.world_normal = prepare_world_normal(
        world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        is_front
    );
    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        #ifdef VERTEX_TANGENTS
        #ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
        #endif
        #endif
        uv,
    );
    pbr_input.V = calculate_view(world_position, pbr_input.is_orthographic);

    return tone_mapping(pbr(pbr_input));
}
