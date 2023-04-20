struct ShaderDataUniforms {
  mouse: vec2<f32>,
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

fn complex_square(z: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(z.x * z.x - z.y * z.y, 2. * z.x * z.y);
}

fn mandelbrot(z: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
    let c2 = dot(c, c);
    // if 256.0 * c2 * c2 - 96.0 * c2 + 32.0 * c.x - 3.0 < 0.0 {
    //     return vec2<f32>(6.9, 420.0);
    // }
    // if 16.0 * (c2 + 2.0 * c.x + 1.0) - 1.0 < 0.0 {
    //     return vec2<f32>(6.9, 420.0);
    // }
    return complex_square(z) + c;
}

fn get_color(uv: vec2<f32>, i: i32, max_i: i32) -> vec4<f32> {
    if i == max_i {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(1.0);
}

fn fractal(C: vec2<f32>) -> vec4<f32> {
    var coords = vec2<f32>(0.0);
    var iter = 0;

    let max_dot = 4.0;

    let max_iteration = 1000;

    while dot(coords, coords) <= max_dot && iter < max_iteration {

        coords = mandelbrot(coords, C);
        iter += 1;
    }

    // if coords == vec2<f32>(69.0, 420.0) {
    //     iter = max_iteration;
    // }

    return get_color(C, iter, max_iteration);
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    //UV ∈ ((-1;-1); (1;1))
    var uv = (in.uv * 2.0) - 1.0;
    uv.x = uv.x * uniforms.aspect.x;
    
    let col = fractal(uv);

    return col * colors[0];
}
