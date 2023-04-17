
struct VertexOutput{
  @builtin(position) position: vec4<f32>,
  @location(0)
  uv: vec2<f32>,
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.uv.x,in.uv.y,0.0,0.0 );
}
