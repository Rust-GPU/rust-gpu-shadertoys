#![cfg_attr(target_arch = "spirv", no_std)]

pub mod shader_prelude;
use shader_prelude::*;
pub mod shaders;
pub mod shared_data;

#[inline(always)]
pub fn fs(constants: &ShaderConstants, mut frag_coord: Vec2) -> Vec4 {
  let resolution = vec3(
    constants.width as f32 as f32,
    constants.height as f32 as f32,
    0.0,
  );
  let time = constants.time;
  let mut mouse = vec4(
    constants.drag_end_x as f32,
    constants.drag_end_y as f32,
    constants.drag_start_x as f32,
    constants.drag_start_y as f32,
  );
  if mouse != Vec4::ZERO {
    mouse.y = resolution.y - mouse.y;
    mouse.w = resolution.y - mouse.w;
  }
  if !(constants.mouse_left_pressed == 1) {
    mouse.z *= -1.0;
  }
  if !(constants.mouse_left_clicked == 1) {
    mouse.w *= -1.0;
  }

  frag_coord.x %= resolution.x;
  frag_coord.y = resolution.y - frag_coord.y % resolution.y;

  let shader_input = ShaderInput {
    resolution,
    time,
    frag_coord,
    mouse,
  };
  let mut shader_output = &mut ShaderResult { color: Vec4::ZERO };
  shaders::render_shader(constants.shader_to_show, &shader_input, &mut shader_output);
  let color = shader_output.color;
  Vec3::powf(color.truncate(), 2.2).extend(color.w)
}

#[allow(unused_attributes)]
#[spirv(fragment)]
pub fn main_fs(
  #[spirv(frag_coord)] in_frag_coord: Vec4,
  #[spirv(push_constant)] constants: &ShaderConstants,
  output: &mut Vec4,
) {
  let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
  let color = fs(constants, frag_coord);
  *output = color;
}

#[allow(unused_attributes)]
#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
  // Create a "full screen triangle" by mapping the vertex index.
  // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
  let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
  let pos = 2.0 * uv - Vec2::ONE;

  *builtin_pos = pos.extend(0.0).extend(1.0);
}
