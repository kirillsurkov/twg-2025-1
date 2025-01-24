#import bevy_render::globals::Globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::sprite_view_bindings::view

@group(0) @binding(1) var<uniform> globals: Globals;

fn to_linear(nonlinear: vec4<f32>) -> vec4<f32> {
    let cutoff = step(nonlinear, vec4<f32>(0.04045));
    let higher = pow((nonlinear + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = nonlinear / vec4<f32>(12.92);
    return mix(higher, lower, cutoff);
}

fn fmod2(a: vec2<f32>, b: f32) -> vec2<f32> {
    return a - b * floor(a / b);
}

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_2(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.0) + 1.0) * x);
}

fn snoise(v: vec2<f32>) -> f32 {
    const C = vec4<f32>(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);

    let i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    let i1 = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), x0.x > x0.y);
    let x12 = x0.xyxy + C.xxzz - vec4<f32>(i1, 0.0, 0.0);

    let i_mod = mod289_2(i);
    let p = permute(permute(i_mod.y + vec3<f32>(0.0, i1.y, 1.0)) + i_mod.x + vec3<f32>(0.0, i1.x, 1.0));

    let m = max(vec3<f32>(0.5) - vec3<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3<f32>(0.0));
    let m2 = m * m;
    let m4 = m2 * m2;

    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    let norm = 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);
    let g = vec3<f32>(
        a0.x * x0.x + h.x * x0.y,
        a0.y * x12.x + h.y * x12.y,
        a0.z * x12.z + h.z * x12.w
    );

    return 130.0 * dot(m4 * norm, g);
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

fn fractal_noise(coord: vec2<f32>, persistence: f32, lacunarity: f32) -> f32 {
    var n = 0.0;
    var frequency = 1.0;
    var amplitude = 1.0;
    for (var o = 0; o < 5; o++) {
        n += amplitude * snoise(coord * frequency);
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    return n;
}


fn fractal_nebula(coord: vec2<f32>, color: vec3<f32>, transparency: f32) -> vec3<f32> {
    return fractal_noise(coord, 0.5, 2.0) * color * transparency;
}

fn dither(color: vec3<f32>, frag_coord: vec2<f32>, levels: f32) -> vec3<f32> {
    var noise = vec3<f32>(dot(vec2(171.0, 231.0), frag_coord.xy));
    noise = fract(noise / vec3(103.0, 71.0, 97.0));
    noise -= 0.5;
    return color + (noise / (levels - 1.0));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * vec2<f32>(view.viewport.zw);
    // let coord = frag_coord / f32(max(view.viewport.z, view.viewport.w));
    let coord2560 = frag_coord / 2560.0;

    let nebula_color1 = hsv2rgb(vec3<f32>(0.66 * cos(globals.time * 0.05), 0.5, 0.25));
    let nebula_color2 = hsv2rgb(vec3<f32>(0.66 * cos(globals.time * 0.07), 1.0, 0.25));

    let nebula1 = fractal_nebula(coord2560 + vec2(globals.time * 0.001 + 0.1, 0.1), nebula_color1, 1.0);
    let nebula2 = fractal_nebula(coord2560 + vec2(globals.time * 0.001, 0.2), nebula_color2, 0.5);

    let stars1 = stars(coord2560 + vec2<f32>(globals.time, 0.0) * 0.004, 8.0, 0.03, 2.0) * vec3(0.74, 0.74, 0.74);
    let stars2 = stars(coord2560 + vec2<f32>(globals.time, 0.0) * 0.002, 16.0, 0.02, 1.0) * vec3(0.97, 0.74, 0.74);
    let stars3 = stars(coord2560 + vec2<f32>(globals.time, 0.0) * 0.001, 32.0, 0.01, 0.5) * vec3(0.9, 0.9, 0.95);

    let result = nebula1 + nebula2 + stars1 + stars2 + stars3;

    return to_linear(vec4<f32>(dither(result, frag_coord, 64.0), 1.0));
}
