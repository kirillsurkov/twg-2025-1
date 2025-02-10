#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

#import noisy_bevy::fbm_simplex_2d

struct BackgroundGlobals {
    time: f32,
    texture_size: vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals: BackgroundGlobals;

fn to_linear(nonlinear: vec3<f32>) -> vec3<f32> {
    let cutoff = step(nonlinear, vec3<f32>(0.04045));
    let higher = pow((nonlinear + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = nonlinear / vec3<f32>(12.92);
    return mix(higher, lower, cutoff);
}

fn fmod2(a: vec2<f32>, b: f32) -> vec2<f32> {
    return a - b * floor(a / b);
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    const K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

fn stars(x: vec2<f32>, num_cells: f32, size: f32, br: f32) -> f32 {
    let n = x * num_cells;
    let f = floor(n);

    var d = 1.0e10;
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            var g = f + vec2(f32(i), f32(j));
            let sz = size * (0.5 + 0.5 * hash12(fmod2(g, num_cells)));
            g = n - g - hash22(fmod2(g, num_cells)) + vec2(hash12(g));
            g /= num_cells * sz;
            d = min(d, dot(g, g));
        }
    }

    return br * smoothstep(0.95, 1.0, 1.0 - sqrt(d));
}

fn fractal_nebula(coord: vec2<f32>, color: vec3<f32>, transparency: f32) -> vec3<f32> {
    return fbm_simplex_2d(coord, 10, 2.0, 0.5, false) * color * transparency;
}

fn dither(color: vec3<f32>, frag_coord: vec2<f32>, levels: f32) -> vec3<f32> {
    var noise = vec3<f32>(dot(vec2(171.0, 231.0), frag_coord.xy));
    noise = fract(noise / vec3(103.0, 71.0, 97.0));
    noise -= 0.5;
    return color + (noise / (levels - 1.0));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    const SPEED = 0.004;

    let globals_time = globals.time;
    let texture_size = globals.texture_size;//vec2<f32>(2560.0, 1440.0);

    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * texture_size;
    // let coord2560 = frag_coord / f32(max(texture_size.x, texture_size.y));
    let coord2560 = frag_coord / 2560.0;

    let nebula_color1 = hsv2rgb(vec3<f32>(0.66 * cos(globals_time * 0.05), 0.5, 0.25));
    let nebula_color2 = hsv2rgb(vec3<f32>(0.66 * cos(globals_time * 0.07), 1.0, 0.25));

    let nebula1 = fractal_nebula(coord2560 + vec2(globals_time * SPEED / 4.0 + 0.1, 0.1), nebula_color1, 1.0);
    let nebula2 = fractal_nebula(coord2560 + vec2(globals_time * SPEED / 4.0, 0.2), nebula_color2, 0.5);

    let stars1 = stars(coord2560 + vec2<f32>(globals_time, 0.0) * SPEED / 1.0, 8.0, 0.03, 2.0) * vec3(0.74, 0.74, 0.74);
    let stars2 = stars(coord2560 + vec2<f32>(globals_time, 0.0) * SPEED / 2.0, 16.0, 0.02, 1.0) * vec3(0.97, 0.74, 0.74);
    let stars3 = stars(coord2560 + vec2<f32>(globals_time, 0.0) * SPEED / 4.0, 32.0, 0.01, 0.5) * vec3(0.9, 0.9, 0.95);

    let result = nebula1 + nebula2 + stars1 + stars2 + stars3;

    return vec4<f32>(to_linear(clamp(dither(result, frag_coord, 64.0), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
