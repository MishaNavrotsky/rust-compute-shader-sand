struct Globals {
  resolution    : vec2<f32>,
  mouse_pos     : vec2<f32>,

  time          : f32,
  cursor_state  : f32,
  _pad0         : vec2<f32>,
};

struct GridCell {
  color : vec4<f32>,
  state : u32,
  flags : u32,
  _pad  : vec2<u32>,
};

@group(0) @binding(0)
var<uniform> globals : Globals;

@group(0) @binding(1)
var<storage, read> grid : array<GridCell>;

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
  return vec4<f32>(globals.mouse_pos.x / 1000, 0.0, 0.0, 1.0);
}