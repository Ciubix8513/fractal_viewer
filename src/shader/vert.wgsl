@vertex
fn main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    if in_vertex_index == 0u {
        return vec4<f32>(-1.0,1.0,0.0,1.0);
    }
    if in_vertex_index == 1u || in_vertex_index == 3u {
        return vec4<f32>(1.0,1.0,0.0,1.0);
    }
    if in_vertex_index == 2u || in_vertex_index == 5u{
        return vec4<f32>(-1.0,-1.0,0.0,1.0);
    }
    if in_vertex_index == 4u {
        return vec4<f32>(1.0,-1.0,0.0,1.0);
    }
  return vec4<f32>(0.0);
}
