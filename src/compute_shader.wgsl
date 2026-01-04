struct Globals {
  resolution : vec2<f32>,
  time       : f32,
  _pad0      : f32,

  mouse_pos  : vec2<f32>,
  _pad1      : vec2<f32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> globals : Globals;

@group(0) @binding(2)
var<storage, read_write> grid : array<u32>;

@group(0) @binding(3)
var<storage, read_write> grid_next : array<u32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let width  = u32(globals.resolution.x);
    let height = u32(globals.resolution.y);

    if (gid.x >= width || gid.y >= height) {
        return;
    }

    let idx = gid.y * width + gid.x;

    grid_next[idx] = 255u;

    let v = grid[idx];

    let r = f32(v);
    textureStore(
        output_tex,
        vec2<i32>(gid.xy),
        vec4(r, 0.0, 0.0, 1.0)
    );
}