#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import noisy_bevy::fbm_simplex_3d

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

fn max2(in: vec2<f32>) -> f32 {
    return max(in.x, in.y);
}

fn linstep(edge1: vec2<f32>, edge2: vec2<f32>, x: vec2<f32>) -> vec2<f32> {
    return clamp((x - edge1) / (edge2 - edge1), vec2<f32>(0.0), vec2<f32>(1.0));
}

fn mortar(uv: vec2<f32>, grid: vec2<f32>, mortar_size: vec2<f32>, mortar_smooth: f32) -> f32 {
    let uv_scaled = uv * grid;
    let mortar_edge1 = 1.0 - 2.0 * mortar_size;
    let mortar_edge2 = 1.0 + 2.0 * mortar_size * (mortar_smooth - 1.0);
    let l = max2(smoothstep(mortar_edge1, mortar_edge2, fract(uv_scaled)));
    let r = max2(1.0 - smoothstep(1.0 - mortar_edge2, 1.0 - mortar_edge1, fract(uv_scaled)));
    return max(l, r);
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn depth(uv: vec2<f32>, size: vec2<f32>, noise_factor: f32) -> f32 {
    let square = max2(smoothstep(vec2<f32>(0.5), vec2<f32>(0.51), abs(uv * 2.0 - 1.0)));
    let m = mortar(uv, vec2<f32>(12.0), vec2<f32>(0.166), 1.5);
    let depth = 1.0 - m;
    let noise = hash12(uv * size);
    return (1.0 - noise_factor) * depth * square + noise_factor * noise;
}

fn normal(uv: vec2<f32>, delta: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    let noise = 0.2;
    let dl = depth(uv - vec2<f32>(delta.x, 0.0), size, noise);
    let dr = depth(uv + vec2<f32>(delta.x, 0.0), size, noise);
    let db = depth(uv - vec2<f32>(0.0, delta.y), size, noise);
    let dt = depth(uv + vec2<f32>(0.0, delta.y), size, noise);
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

    let depth = depth(in.uv, size, 0.0);
    let normal = normal(in.uv, delta, size);
    let facing = smoothstep(0.9, 1.0, depth);
    let br = facing * smoothstep(0.95, 1.0, fbm_simplex_3d(vec3<f32>(floor(in.uv * 12.0), material.seed + material.time * 0.01), 2, 2.0, 2.0, false));

    let color = vec4<f32>(vec3<f32>(0.0), 1.0);
    let emissive = vec4<f32>(br * hsv2rgb(vec3<f32>(fract(material.seed), 1.0, 1.0)) * 5.0, 1.0);
    let metallic = facing;
    let roughness = 1.0;

    textureStore(out_color, frag_coord, in.index, color);
    textureStore(out_emissive, frag_coord, in.index, emissive);
    textureStore(out_metallic, frag_coord, in.index, vec4<f32>(metallic, 0.0, 0.0, 0.0));
    textureStore(out_roughness, frag_coord, in.index, vec4<f32>(roughness, 0.0, 0.0, 0.0));
    textureStore(out_normal, frag_coord, in.index, vec4<f32>(normal, 0.0));
}
