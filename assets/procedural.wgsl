#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT
#import bevy_pbr::pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing, apply_normal_mapping, calculate_tbn_mikktspace, calculate_view, prepare_world_normal}
#import bevy_pbr::forward_io::VertexOutput
#import bevy_core_pipeline::oit::oit_draw

@group(2) @binding(100) var color_texture: texture_2d_array<f32>;
@group(2) @binding(101) var color_texture_sampler: sampler;
@group(2) @binding(102) var emissive_texture: texture_2d_array<f32>;
@group(2) @binding(103) var emissive_texture_sampler: sampler;
@group(2) @binding(104) var metallic_texture: texture_2d_array<f32>;
@group(2) @binding(105) var metallic_texture_sampler: sampler;
@group(2) @binding(106) var roughness_texture: texture_2d_array<f32>;
@group(2) @binding(107) var roughness_texture_sampler: sampler;
@group(2) @binding(108) var normal_texture: texture_2d_array<f32>;
@group(2) @binding(109) var normal_texture_sampler: sampler;
@group(2) @binding(150) var<uniform> index: u32;
@group(2) @binding(151) var<uniform> add_emission: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.N = calculate_tbn_mikktspace(in.world_normal, in.world_tangent) * textureSample(normal_texture, normal_texture_sampler, in.uv, index).xyz;
    pbr_input.material.base_color *= textureSample(color_texture, color_texture_sampler, in.uv, index);
    pbr_input.material.emissive *= textureSample(emissive_texture, emissive_texture_sampler, in.uv, index);
    pbr_input.material.emissive += add_emission;
    pbr_input.material.metallic = textureSample(metallic_texture, metallic_texture_sampler, in.uv, index).r;
    pbr_input.material.perceptual_roughness = textureSample(roughness_texture, roughness_texture_sampler, in.uv, index).r;
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var color = apply_pbr_lighting(pbr_input);
    color = main_pass_post_lighting_processing(pbr_input, color);
    color += vec4<f32>(pbr_input.material.emissive.rgb, 0.0);

    oit_draw(in.position, color);
}
