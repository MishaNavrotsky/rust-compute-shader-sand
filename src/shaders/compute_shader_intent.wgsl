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
var<storage, read_write> intent: array<Intent>;

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

  intent[idx].kind = INTENT_STAY;
  intent[idx].flags = 0;

  if (grid[idx].state != CELL_SAND) {
    return;
  }

  if (y + 1u >= h) {
    return;
  }

  let down = grid[get_idx(x, y + 1u, w)].state == CELL_EMPTY;
  let dl = x > 0u && grid[get_idx(x - 1u, y + 1u, w)].state == CELL_EMPTY;
  let dr = x + 1u < w && grid[get_idx(x + 1u, y + 1u, w)].state == CELL_EMPTY;

  if (down) {
    intent[idx].kind = INTENT_MD;
  }
  else {
    let flip = ((x + y + globals.frame) & 1u) == 0u;

    if (flip && dl) {
      intent[idx].kind = INTENT_MDL;
    }
    else if (!flip && dr) {
      intent[idx].kind = INTENT_MDR;
    }
    else if (dl) {
      intent[idx].kind = INTENT_MDL;
    }
    else if (dr) {
      intent[idx].kind = INTENT_MDR;
    }
  }
}