struct Globals {
  resolution : vec2<f32>, // [f32; 2]
  time       : f32,
  _pad0      : f32,        // padding to 16 bytes

  mouse_pos  : vec2<f32>,  // [f32; 2]
  _pad1      : vec2<f32>,  // padding
};

@group(0) @binding(0)
var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1)
var<uniform> globals: Globals;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(output_tex);

    let res = vec2<u32>(globals.resolution);
    if (any(gid.xy >= res)) {
        return;
    }

    let pulse = 0.5 + 0.5 * sin(globals.time);
    let R = mix(20.0, 100.0, pulse);
    
    let d = distance(vec2<f32>(gid.xy) + 0.5, globals.mouse_pos);
    
    let inside = 1.0 - smoothstep(0.0, R, d);


    let color = vec4<f32>(
        inside,
        0,
        0,
        1,
    );

    textureStore(output_tex, vec2<i32>(gid.xy), color);
}