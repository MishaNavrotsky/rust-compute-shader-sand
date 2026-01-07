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

@group(0) @binding(2)
var<storage, read_write> grid_next: array<GridCell>;

@group(0) @binding(3)
var<storage, read> intent: array<Intent>;

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

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let w = globals.resolution.x;
  let h = globals.resolution.y;

  let x = gid.x;
  let y = gid.y;

  if (x >= w || y >= h) {
    return;
  }

  let idx = get_idx(x, y, w);

  grid_next[idx].state = CELL_EMPTY;
  grid_next[idx].color = vec4f(0.0);
  grid_next[idx].flags = 0u;

  if (y > 0u) {
    let src = get_idx(x, y - 1u, w);
    if (grid[src].state == CELL_SAND && intent[src].kind == INTENT_MD) {
      grid_next[idx] = grid[src];
      return;
    }
  }

  if (x > 0u && y > 0u) {
    let src = get_idx(x - 1u, y - 1u, w);
    if (grid[src].state == CELL_SAND && intent[src].kind == INTENT_MDR) {
      grid_next[idx] = grid[src];
      return;
    }
  }

  if (x + 1u < w && y > 0u) {
    let src = get_idx(x + 1u, y - 1u, w);
    if (grid[src].state == CELL_SAND && intent[src].kind == INTENT_MDL) {
      grid_next[idx] = grid[src];
      return;
    }
  }

  if (grid[idx].state == CELL_SAND && intent[idx].kind == INTENT_STAY) {
    grid_next[idx] = grid[idx];
    return;
  }

  let p = vec2<f32>(gid.xy) + vec2<f32>(0.5);
  let d = p - globals.mouse_pos;
  let inside = dot(d, d) < 50.0;

  if (inside && globals.mouse_state == MOUSE_PRESSED) {
    if (grid_next[idx].state == CELL_EMPTY) {
      grid_next[idx].state = CELL_SAND;
      grid_next[idx].color = vec4<f32>(0.82, 0.76, 0.52, 0.1);
    }
  }
}