struct Globals {
  resolution: vec2<u32>,
  mouse_pos: vec2<f32>,

  time: f32,
  mouse_state: u32,
  frame: u32,
  _pad0: f32,
}

;

struct GridCell {
  color: vec4<f32>,
  state: u32,
  flags: u32,
  _pad: vec2<u32>,
}

;

struct Intent {
  kind: u32,
  flags: u32,
}

;

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var<storage, read> grid: array<GridCell>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let pos = array<vec2<f32>, 3>(vec2<f32>(- 1.0, - 1.0), vec2<f32>(3.0, - 1.0), vec2<f32>(- 1.0, 3.0),);

  return vec4<f32>(pos[i], 0.0, 1.0);
}

fn get_idx(x: u32, y: u32, width: u32) -> u32 {
  return y * width + x;
}

const INTENT_STAY: u32 = 0;
const INTENT_MD: u32 = 1;
const INTENT_MDL: u32 = 2;
const INTENT_MDR: u32 = 3;

const CELL_EMPTY: u32 = 0;
const CELL_SAND: u32 = 1;

const MOUSE_DEFAULT = 0;
const MOUSE_PRESSED = 1;

const MOVE_DOWN: u32 = 1u << 0;
const MOVE_DL: u32 = 1u << 1;
const MOVE_DR: u32 = 1u << 2;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let w = globals.resolution.x;
  let h = globals.resolution.y;

  let xi = i32(floor(pos.x));
  let yi = i32(floor(pos.y));

  if (xi < 0 || yi < 0 || xi >= i32(w) || yi >= i32(h)) {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }

  let x = u32(xi);
  let y = u32(yi);
  let idx = get_idx(x, y, w);

  return grid[idx].color;
}
