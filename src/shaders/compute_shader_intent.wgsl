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

struct Intent {
  kind  : u32,
  flags : u32,
};

@group(0) @binding(0)
var<uniform> globals : Globals;

@group(0) @binding(1)
var<storage, read> grid : array<GridCell>;

@group(0) @binding(2)
var<storage, read_write> intent : array<Intent>;



@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let w = u32(globals.resolution.x);
  let h = u32(globals.resolution.y);

  let x = gid.x;
  let y = gid.y;

  if (x >= w || y >= h) {
      return;
  }
}