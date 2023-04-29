struct ShaderDataUniforms {
  position: vec2<f32>,
  resolution: vec2<u32>,
  aspect: vec2<f32>,
  zoom: f32,
  arr_len: i32,
  fractal: u32,
  max_iter: u32,
  color_num: u32,
  msaa: u32,
}

@group(0)
@binding(0)
var<uniform> uniforms : ShaderDataUniforms;

@group(0)
@binding(1)
var<storage, read>  colors : array<vec4<f32>>;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0)
  uv: vec2<f32>,
}

//I don't remember where I got this, but it should work
fn rand(s: f32) -> f32 {
    return fract(sin(s * 12.9898) * 43758.5453);
}

fn complex_square(z: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}
fn complex_cube(z: vec2<f32>) -> vec2<f32> {
    let x2 = z.x * z.x;
    let y2 = z.y * z.y;
    return vec2<f32>(z.x * x2 - 3.0 * z.x * y2, 3.0 * x2 * z.y - z.y * y2);
}
fn complex_pow(z: vec2<f32>, n: f32) -> vec2<f32> {
    let r = length(z);
    let theta = atan2(z.y, z.x);
    return pow(r, n) * vec2(cos(n * theta), sin(n * theta));
}
fn complex_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let denumenator = 1.0 / (b.x * b.x + b.y * b.y);
    //Multiplying should be a bit faster
    return vec2<f32>((a.x * b.x + a.y * b.y) * denumenator, (a.y * b.x - a.x * b.y) * denumenator);
}
fn complex_sqrt(z: vec2<f32>) -> vec2<f32> {
    let l = length(z) * 0.5;
    let x = z.x * .5;
    //I don't think I need this much optimiazation, but better safe than sorry
    return vec2(sqrt(l + x), sign(z.y) * sqrt(l - x));
} 

fn mandelbrot(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let c2 = dot(c, c);

    // skip computation inside M1 - https://iquilezles.org/articles/mset1bulb
    if 256.0 * c2 * c2 - 96.0 * c2 + 32.0 * c.x - 3.0 < 0.0 {
        return vec2<f32>(6.9, 420.0);
    }
    // skip computation inside M2 - https://iquilezles.org/articles/mset2bulb
    if 16.0 * (c2 + 2.0 * c.x + 1.0) - 1.0 < 0.0 {
        return vec2<f32>(6.9, 420.0);
    }
    return complex_square(z) + c;
}
fn burning_ship(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let z1 = abs(z);
    return complex_square(z1) + c;
}
fn tricorn(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let z1 = z * vec2<f32>(1.0, -1.0);
    return complex_square(z1) + c;
}
fn feather(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    return complex_div(complex_cube(z), (vec2<f32>(1.0, 0.0) + (z * z))) + c;
}
fn eye(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    if length(c) > 5.0 {
        return vec2<f32>(6.9, 420.0);
    }
    return complex_square(complex_div(z, c)) + c;
}

fn get_col(coord: f32, col_num: i32) -> vec4<f32> {
    if col_num == 1 {
        return colors[0];
    }
    let cstep1 = 1.0 / f32(col_num - 1);
    for (var i = 1; i < col_num; i += 1) {
        if coord < cstep1 * f32(i) {
            return mix(colors[(i - 1) % uniforms.arr_len], colors[i % uniforms.arr_len], coord / cstep1 - f32(i - 1));
        }
    }
    return vec4<f32>(coord);
}

fn get_color(uv: vec2<f32>, i: f32, max_i: u32) -> vec4<f32> {
    if i >= f32(max_i) {
        return vec4<f32>(0.0);
    }
    return get_col(f32(i) / f32(max_i), i32(uniforms.color_num));
}

fn fractal(C: vec2<f32>) -> vec4<f32> {
    var coords = vec2<f32>(0.0);
    var iter = 0u;

    var max_dot = 5.0;
    if (uniforms.fractal * 8u) == 8u || (uniforms.fractal & 16u) == 16u {max_dot = 2000.0;}
    let max_iteration = uniforms.max_iter;

    while dot(coords, coords) <= max_dot && iter < max_iteration {

        if (uniforms.fractal & 1u) == 1u {
            coords = mandelbrot(coords, C);
        } else if (uniforms.fractal & 2u) == 2u {
            coords = burning_ship(coords, C);
        } else if (uniforms.fractal & 4u) == 4u {
            coords = tricorn(coords, C);
        } else if (uniforms.fractal & 8u) == 8u {
            coords = feather(coords, C);
        } else if (uniforms.fractal & 16u) == 16u {
            coords = eye(coords, C);
        }
        iter += 1u;
    }
    var i = f32(iter);
    if coords.x == 6.9 && coords.y == 420.0 {
        i = f32(max_iteration);
    } else if (uniforms.fractal & 2147483648u) != 0u {
        i = i - log2(log2(dot(coords, coords))) + 4.0;
    }
    return get_color(C, i, max_iteration);
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // UV ∈ ((-1;-1); (1;1))
    var uv = (in.uv * 2.0) - 1.0;
    uv.x = (uv.x * uniforms.aspect.x);
    uv = uv - uniforms.position;
    uv = uv / uniforms.zoom;

    if length(abs(uv) - vec2<f32>(1.0, 1.0)) < .1 {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    var col = vec4<f32>(0.0);
    let msaa = f32(uniforms.msaa);

    for (var i = 0.0; i < msaa; i += 1.0) {
        let dxy = vec2<f32>(rand(i * .54321), rand(i * .12345)) / 1000.0;
        let c = (uv + dxy) * vec2<f32>(1.0, -1.0);
        col += fractal(c);
    }

    return col / msaa;
}
