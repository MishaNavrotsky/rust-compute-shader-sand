struct Globals {
  resolution    : vec2<u32>,
  mouse_pos     : vec2<f32>,
  
  time          : f32,
  mouse_state  : u32,
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


fn get_idx(x: u32, y: u32, width: u32) -> u32 {
  return y * width + x;
}


@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let w = globals.resolution.x;
  let h = globals.resolution.y;
  
  let x = gid.x;
  let y = gid.y;

  if (x >= w || y >= h) {
      return;
  }
  
  let idx = get_idx(x, y, w);

  intent[idx].flags = grid[idx].flags;
}