struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct CustomMaterial {
    // Needed for 16-bit alignment on WebGL2
    time: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: CustomMaterial;
@group(2) @binding(1) var height_texture: texture_2d<f32>;
@group(2) @binding(2) var height_texture_sampler: sampler;
@group(2) @binding(3) var waves_texture: texture_2d<f32>;
@group(2) @binding(4) var waves_texture_sampler: sampler;


const UV_STEPS = 64.0;

fn rgb(r: i32, g: i32, b: i32) -> vec3<f32> {
    return vec3<f32>(f32(r) / 255.0, f32(g) / 255.0, f32(b) / 255.0);
}

fn quantize(v: f32, n: f32) -> f32 {
    return (floor(v * n) + 0.5) / n;
}

fn quantize_min(v: f32, n: f32) -> f32 {
    return (floor(v * n)) / n;
}
fn quantize_max(v: f32, n: f32) -> f32 {
    return (ceil(v * n)) / n;
}

fn v2(x: f32, y: f32) -> vec2<f32> {
    return vec2<f32>(x, y);
}

fn rot(v: vec2<f32>, a: f32) -> vec2<f32> {
    return v * mat2x2<f32>(cos(a), -sin(a), sin(a), cos(a));
}

fn h(p: vec2<f32>) -> f32 {
    return textureSample(height_texture, height_texture_sampler, v2(p.x, 1.0 - p.y)).r;
}

fn w(p: vec2<f32>) -> vec2<f32> {
    return textureSample(waves_texture, waves_texture_sampler, v2(p.x, 1.0 - p.y)).rg;
}

fn wave_height(p: vec2<f32>, uv: vec2<f32>, t: f32) -> f32 {
    let dir_global = w(uv);
    var dir = normalize(dir_global);

    let v = dot((p*0.05), dir);

    let t1 = sin(v + t*5.5);

    return t1;
    // return dir.y;
}

// top, left, right, bottom
fn surrounding_heights(uv: vec2<f32>) -> vec4<f32> {
    let x = quantize(uv.x, UV_STEPS);
    let y = quantize(uv.y, UV_STEPS);
    let offset = 1.0 / UV_STEPS;

    let top = y + offset;
    let bottom = y - offset;
    let left = x - offset;
    let right = x + offset;

    return vec4<f32>(
        h(v2(x, top)),
        h(v2(left, y)),
        h(v2(right, y)),
        h(v2(x, bottom))
    );
}

// this should interpolate between the closest 4 pixels
fn bilinear_height(uv: vec2<f32>) -> f32 {
    let texel = 1.0 / UV_STEPS;

    let uv_tex = uv * UV_STEPS;

    let i0 = floor(uv_tex);
    let i1 = i0 + vec2<f32>(1.0, 1.0);

    let tx = fract(uv_tex.x);
    let ty = fract(uv_tex.y);

    let uv00 = (i0) * texel;
    let uv10 = vec2<f32>(i1.x, i0.y) * texel;
    let uv01 = vec2<f32>(i0.x, i1.y) * texel;
    let uv11 = (i1) * texel;

    let h00 = h(uv00);
    let h10 = h(uv10);
    let h01 = h(uv01);
    let h11 = h(uv11);

    let hx0 = mix(h00, h10, tx);
    let hx1 = mix(h01, h11, tx);
    let hxy = mix(hx0, hx1, ty);

    return h(uv);

    // return hxy;
}

// returning (distance, height)
fn distance_to_closest_neighbor(uv: vec2<f32>) -> vec2<f32> {
    let x = quantize(uv.x, UV_STEPS) - uv.x;
    let y = quantize(uv.y, UV_STEPS) - uv.y;


    return v2(length(v2(x, y)), 1.0);
}

// height between -1.0..1.0
fn water_color(block_height: f32, height: f32, uv: vec2<f32>) -> vec4<f32> {

    var c = rgb(99, 115, 229);
    let depth_modifier = 1.0 + height;

    let saturation = clamp(-height, 0.0, 1.0);

    return vec4<f32>(c, saturation) ;
            // return vec4<f32>(1.0) * d;
        // }

        // c *= depth_modifier;

        // return vec4<f32>(c, 1.0);
}

fn ground_color(block_height: f32, height: f32, uv: vec2<f32>) -> vec3<f32> {
    return rgb(121, 115, 45);
}


fn merge_colors(water_color: vec4<f32>, ground_color: vec3<f32> ) -> vec3<f32> {
    return mix(ground_color, water_color.xyz, water_color.a);
}

fn shore_dist(uv: vec2<f32>, h: f32) -> f32 {

    let s = surrounding_heights(uv);


    return 1.0;
}



@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let block_height = h(mesh.uv);
    let local_height = bilinear_height(mesh.uv);

    let water = water_color(block_height, local_height, mesh.uv);

    let ground = ground_color(block_height, local_height, mesh.uv);

    var c: vec3<f32>;
    if block_height < 0.5 {
        c = vec3<f32>(0.0, 0.0, 1.0) * (wave_height(mesh.world_position.xy, mesh.uv, material.time.x) + 1.0)*0.5;
    } else if block_height == 0.5 {
        c = vec3<f32>(0.5);
    } else {
        c = vec3<f32>(0.0, 1.0, 0.0);
    }

    return vec4<f32>(c * abs(block_height), 1.0);
    // return vec4<f32>((normalize(w(mesh.uv))+1.0),0.0, 1.0);
}
