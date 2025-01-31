#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import noisy_bevy::fbm_simplex_2d_seeded

struct ProceduralMaterialGlobals {
    texture_size: vec2<f32>,
}

struct RoomFloorMaterial {
    seed: f32,
    time: f32,
    time_multiplier: f32,
}

@group(0) @binding(0) var out_color: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(1) var out_emissive: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(2) var out_metallic: texture_storage_2d_array<r8unorm, write>;
@group(0) @binding(3) var out_roughness: texture_storage_2d_array<r8unorm, write>;
@group(0) @binding(4) var out_normal: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(5) var<uniform> globals: ProceduralMaterialGlobals;
@group(0) @binding(100) var<storage, read> material: array<RoomFloorMaterial>;

fn to_linear(nonlinear: vec3<f32>) -> vec3<f32> {
    let cutoff = step(nonlinear, vec3<f32>(0.04045));
    let higher = pow((nonlinear + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = nonlinear / vec3<f32>(12.92);
    return mix(higher, lower, cutoff);
}

fn depth(uv: vec2<f32>, seed: f32) -> f32 {
    return fbm_simplex_2d_seeded(uv * 30.0, 3, 2.0, 0.5, 0.0, true);
}

fn normal(uv: vec2<f32>, delta: vec2<f32>, seed: f32) -> vec3<f32> {
    let noise = 0.2;
    let dl = depth(uv - vec2<f32>(delta.x, 0.0), seed);
    let dr = depth(uv + vec2<f32>(delta.x, 0.0), seed);
    let db = depth(uv - vec2<f32>(0.0, delta.y), seed);
    let dt = depth(uv + vec2<f32>(0.0, delta.y), seed);
    return normalize(vec3<f32>(dl - dr, db - dt, 2.0));
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    const K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
    @location(1)
    index: u32,
};

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return VertexOutput(clip_position, uv, instance_index);
}

@fragment
fn fragment(in: VertexOutput) {
    let material = material[in.index];

    let delta = dpdx(in.uv) + dpdy(in.uv);
    let size = globals.texture_size;
    let frag_coord = vec2<i32>(in.uv * size);

    let seed = material.seed;

    let depth = smoothstep(0.1, 0.9, depth(in.uv, seed));
    let normal = normal(in.uv, delta, seed);

    let silicon = vec3<f32>(0.4, 0.35, 0.3);
    let ice = vec3<f32>(0.4, 0.35, 0.3);

    let color = vec4<f32>(depth * silicon, 1.0);
    // let emissive = vec4<f32>(br * hsv2rgb(vec3<f32>(fract(material.seed), 1.0, 1.0)) * 5.0, 1.0);
    let metallic = depth;
    let roughness = 1.0 - depth;

    textureStore(out_color, frag_coord, in.index, color);
    // textureStore(out_emissive, frag_coord, in.index, emissive);
    textureStore(out_metallic, frag_coord, in.index, vec4<f32>(metallic, 0.0, 0.0, 0.0));
    textureStore(out_roughness, frag_coord, in.index, vec4<f32>(roughness, 0.0, 0.0, 0.0));
    textureStore(out_normal, frag_coord, in.index, vec4<f32>(normal, 0.0));
}
