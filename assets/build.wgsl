#import bevy_pbr::mesh_view_bindings::globals
#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::pbr_functions::alpha_discard

#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput}
#import bevy_pbr::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing}

#import bevy_core_pipeline::oit::oit_draw

#import noisy_bevy::fbm_simplex_3d

struct BuildMaterialSettings {
    created: f32,
    color: vec4<f32>,
    direction: f32,
}

@group(2) @binding(100)
var<uniform> material_settings: BuildMaterialSettings;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) {
    const DURATION = 3.0;
    const ABOVE = 1.0;
    const BELOW = 0.2;
    const HEIGHT = ABOVE + BELOW;
    const LINE = 0.2;

    let sample_from = -material_settings.direction;
    let sample_to = material_settings.direction;

    var elapsed = min(1.0, (globals.time - material_settings.created) / DURATION);
    elapsed = sample_from + (sample_to - sample_from) * elapsed;
    elapsed = elapsed * 0.5 + 0.5;

    let noise = mix(fbm_simplex_3d(in.world_position.xyz, 2, 4.0, 4.0, false) * LINE * 0.5, 1.0, elapsed);

    let edge = (HEIGHT + LINE) * noise - BELOW;
    var alpha = 1.0 - smoothstep(edge - LINE * 0.5, edge + LINE * 0.5, in.world_position.z);
    var highlight = smoothstep(edge + LINE, edge - LINE, in.world_position.z);
    highlight = alpha * (1.0 - min(highlight, 1.0 - highlight));

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var emissive = mix(material_settings.color.rgb, vec3<f32>(0.0, 0.0, 0.0), highlight) * 100.0;
    pbr_input.material.base_color = mix(vec4<f32>(material_settings.color.rgb, 0.0), pbr_input.material.base_color, alpha);

    var color = apply_pbr_lighting(pbr_input);
    color += vec4<f32>(emissive, 0.0);
    color += vec4<f32>(material_settings.color.rgb * 0.05, 0.0);

    oit_draw(in.position, color);
}
