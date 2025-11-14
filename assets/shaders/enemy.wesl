struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct CustomMaterial {
    // Needed for 16-bit alignment on WebGL2
    time: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: CustomMaterial;



fn rgb(r: i32, g: i32, b: i32) -> vec3<f32> {
    return vec3<f32>(f32(r) / 255.0, f32(g) / 255.0, f32(b) / 255.0);
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

fn mast_d(p: vec2<f32>) -> f32 {
    return length(p) - 0.1;
}

fn pie(p: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
    var localp = p;
    localp.x = abs(localp.x);
    let l = length(p) - r;
    let m = length(localp - c*clamp(dot(localp, c), 0.0, r));
    return max(l, m*sign(c.y*localp.x- c.x*localp.y));
}

fn sail_d(p: vec2<f32>) -> f32 {
    return pie(p*vec2(1.0, 4.4), vec2(-1.0, 0.5), 1.6);
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let brown = rgb(123, 84, 47);

    let p = mesh.uv*10.0 -vec2<f32>(5.0, 5.0);

    var a: f32 ;
    if chamfer(p, vec2<f32>(0.7, 1.5), 0.7) < 0.0 {
        a = 1.0;
    } else {
        a = 0.0;
    }
    var wood =vec4(brown, a);

    if mast_d(p) < 0.0 {
        wood = vec4(0.0, 0.0, 0.0, 1.0);
    }

    if sail_d(p) < 0.0 {
        wood = vec4(1.0);
    }




    return wood;
}
