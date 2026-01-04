@group(0) @binding(0)
var input_tex: texture_storage_2d<rgba8unorm, read>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );

    return vec4<f32>(pos[i], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = vec2<i32>(pos.xy);
  let color = textureLoad(input_tex, pixel);
  return vec4<f32>(color.rgb, 1.0);
}