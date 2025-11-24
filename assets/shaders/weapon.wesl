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


const pi = 3.141592653589793238462643;

fn rgb(r: i32, g: i32, b: i32) -> vec3<f32> {
    return vec3<f32>(f32(r) / 255.0, f32(g) / 255.0, f32(b) / 255.0);
}

fn rot(p: vec2f, a: f32) -> vec2f {
    return p * mat2x2<f32>(cos(a), sin(a), -sin(a), cos(a));
}

fn chamfer(p: vec2<f32>, b: vec2<f32>, chamfer: f32) -> f32 {
    var local_p = abs(p) - b;

    if local_p.y > local_p.x {
        local_p = local_p.yx;
    }
    local_p.y += chamfer;

    let k = 1.0 - sqrt(2.0);
    if local_p.y < 0.0 && (local_p.y + local_p.x * k) < 0.0 {
        return local_p.x;
    }

    if local_p.x < local_p.y {
        return (local_p.x + local_p.y) * sqrt(0.5);
    }
    return length(local_p);
}

fn sdBox(p: vec2<f32>, center: vec2<f32>, size: vec2<f32>) -> f32 {
    let d = abs(p - center) - size * 0.5;
    return length(max(d, vec2<f32>(0.0, 0.0))) + min(max(d.x, d.y), 0.0);
}
fn sdTriangleIsosceles(p: vec2f, q: vec2f) -> f32 {
    var local_p = p;
    local_p.x = abs(local_p.x);
    let a = local_p - q * clamp(dot(local_p, q) / dot(q, q), 0.0, 1.0);
    let b = local_p - q * vec2(clamp(local_p.x / q.x, 0.0, 1.0), 1.0);
    let s = -sign(q.y);
    let d = min(vec2(dot(a, a), s * (local_p.x * q.y - local_p.y * q.x)),
        vec2(dot(b, b), s * (local_p.y - q.y)));
    return -sqrt(d.x) * sign(d.y);
}

fn mast_d(p: vec2<f32>) -> f32 {
    return length(p) - 0.1;
}

fn pie(p: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
    var localp = p;
    localp.x = abs(localp.x);
    let l = length(p) - r;
    let m = length(localp - c * clamp(dot(localp, c), 0.0, r));
    return max(l, m * sign(c.y * localp.x - c.x * localp.y));
}

fn sail_d(p: vec2<f32>) -> f32 {
    return pie(p * vec2(1.0, 4.4), vec2(-1.0, 0.5), 1.6);
}

fn cannon_ball(
    mesh: VertexOutput
) -> vec4<f32> {
    let iron = rgb(52, 52, 50);
    let p = mesh.uv * 10.0 - vec2<f32>(5.0, 5.0);

    var color: vec4<f32>;

    if length(p) < 3.0 {
        color = vec4(iron, 1.0);
    } else {
        color = vec4(0.0);
    }
    return color;
}

fn rotationMatrix(angles: vec3<f32>) -> mat3x3<f32> {
    let cx = cos(angles.x);
    let sx = sin(angles.x);
    let cy = cos(angles.y);
    let sy = sin(angles.y);
    let cz = cos(angles.z);
    let sz = sin(angles.z);

    return mat3x3<f32>(
        vec3<f32>( cy * cz,                   -cy * sz,                  sy       ),
        vec3<f32>( sx * sy * cz + cx * sz,   -sx * sy * sz + cx * cz,  -sx * cy  ),
        vec3<f32>( -cx * sy * cz + sx * sz,   cx * sy * sz + sx * cz,   cx * cy  )
    );
}

fn dotnoise(uv_in: vec3<f32>, res: f32) -> f32 {
    var uv = uv_in;
    var v: f32 = 0.0;

    // WGSL requires a constant loop bound and uses 'var' for the index
    for (var i: i32 = 0; i < 4; i = i + 1) {
        // Note: 0.002 is promoted to f32, res is f32 already
        uv = uv * rotationMatrix(vec3<f32>(fract(sin(uv / res * 0.002))));
        v = v + dot(cos(uv * 200.0), sin(uv.yzx * 100.0)) / res * 16.0;
    }

    return v / 50.0;
}

fn billow(p: vec3f) -> f32 {
    let lunacricy = 2.0;
    let dampening = 0.5;
    var a = 1.0;
    var t = 0.0;
    for (var i = 0; i < 3; i = i + 1) {
        t += abs(a*dotnoise(f32(i) * lunacricy * p, 1.0));
        a *= dampening;

    }

    return t;
}

fn flame_strike(
    mesh: VertexOutput
) -> vec4<f32> {
    let orange =rgb(141, 74, 17);
    let scale = mix(10.0, 1.0, material.time.x);
    let p = mesh.uv * scale - vec2<f32>(scale*0.5);


    let len2p = length(2.0 * p);
    var color = 3.0 - (3.0 * len2p);

    var coord = vec3<f32>(
        atan2(p.x, p.y) / 6.2832 + 0.5,
        length(p) * 0.4,
        0.5
    );

    for (var i: i32 = 1; i <= 3; i = i + 1) {
        let power = pow(2.0, f32(i));
        color = color + (1.5 / power) *
            dotnoise(
                coord + vec3<f32>(0.0, -material.time.x * 0.05, material.time.x * 0.01),
                power * 16.0
            );
    }

    let c = max(color, 0.0);
    return vec4<f32>(
        color,
        pow(c, 2.0) * 0.4,
        pow(c, 3.0) * 0.15,
        c
    );
}

fn arrow(
    mesh: VertexOutput
) -> vec4<f32> {
    let p = (mesh.uv.yx - vec2f(0.5));
    let wood = rgb(130, 83, 42);
    let iron = rgb(84, 94, 114);

    let shaftCenter = vec2f(0.0, 0.0);
    let shaftSize = vec2f(0.5, 0.12); // length, thickness


    var shaft = sdBox(p, shaftCenter, shaftSize);
    for (var i: f32 = 0.0; i < 0.3; i = i + 0.1) {
        let fledge = sdTriangleIsosceles(rot(p-vec2f(i, 0.0), pi*-0.5)*.3, vec2(0.05));
        shaft = min(shaft, fledge);

    }
    let head = sdTriangleIsosceles(rot(p+vec2f(0.5, 0.0), pi*-0.5)*.3, vec2(0.05));

    if min(shaft, head) > 0.0 {
        return vec4(0.0);
    }

    if shaft < head {
        return vec4(wood, 1.0);
    } else {
        return vec4(iron, 1.0);
    }
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    if material.time.y == 1.0 {
        return flame_strike(mesh);
    } else if material.time.y == 2.0 {
        return arrow(mesh);
    } else {
        return cannon_ball(mesh);
    }
}
