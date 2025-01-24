#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import noisy_bevy::fbm_simplex_3d

struct RoomFloorMaterial {
    time: f32,
    seed: f32,
}

@group(0) @binding(0) var<uniform> material: RoomFloorMaterial;

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
    // let m1 = mortar(uv, vec2<f32>(4.0), vec2<f32>(0.06), 1.5);
    let m2 = mortar(uv, vec2<f32>(12.0), vec2<f32>(0.166), 1.5);
    // let depth = 1.0 - max(m1, m2);
    let depth = 1.0 - m2;
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

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>,
    @location(2) metallic_roughness: vec4<f32>,
    @location(3) normal: vec4<f32>,
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> FragmentOutput {
    var res = FragmentOutput();

    let delta = dpdx(in.uv) + dpdy(in.uv);
    let size = 1.0 / delta;

    let depth = depth(in.uv, size, 0.0);
    let normal = normal(in.uv, delta, size);
    let facing = smoothstep(0.9, 1.0, depth);
    let br = facing * smoothstep(0.95, 1.0, fbm_simplex_3d(vec3<f32>(floor(in.uv * 12.0), material.seed + material.time * 0.01), 2, 2.0, 2.0));

    res.color = vec4<f32>(vec3<f32>(0.1), 1.0);
    res.emissive = vec4<f32>(br * hsv2rgb(vec3<f32>(fract(material.seed), 1.0, 1.0)) * 5.0, 1.0);
    res.metallic_roughness = vec4<f32>(0.0, 1.0 - facing, facing, 0.0) * 0.2 + vec4<f32>(0.0, 0.1, 0.8, 0.0);
    res.normal = vec4<f32>(normal, 0.0);

    return res;
}
