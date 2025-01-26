#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::pbr_functions::alpha_discard

#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput}
#import bevy_pbr::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}

#import bevy_core_pipeline::oit::oit_draw

struct MyExtendedMaterial {
    created: f32,
}

@group(2) @binding(100)
var<uniform> my_extended_material: MyExtendedMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) {
    let elapsed = globals.time - my_extended_material.created;

    const DURATION = 3.0;
    const ABOVE = 1.0;
    const BELOW = 0.5;
    const HEIGHT = ABOVE + BELOW;
    const LINE = 0.2;

    let edge = (HEIGHT + LINE) * elapsed / DURATION - BELOW;
    let alpha = 1.0 - smoothstep(edge - LINE * 0.5, edge + LINE * 0.5, in.world_position.z);
    var highlight = smoothstep(edge + LINE, edge - LINE, in.world_position.z);
    highlight = min(highlight, 1.0 - highlight);

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.emissive = mix(vec4<f32>(0.0, 30.0, 0.0, 0.1), vec4<f32>(0.0), alpha);
    pbr_input.material.base_color = mix(vec4<f32>(0.0, 10.0, 0.0, 0.2), pbr_input.material.base_color, alpha);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var color = apply_pbr_lighting(pbr_input);

    color = mix(color, vec4<f32>(0.0, 1.0, 0.0, 1.0), 0.02);
    oit_draw(in.position, color);
}
