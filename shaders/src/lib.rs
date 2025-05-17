#![cfg_attr(target_arch = "spirv", no_std)]

use shared::*;
use spirv_std::{
  glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4},
  spirv,
};

pub mod a_lot_of_spheres;
pub mod a_question_of_time;
pub mod apollonian;
pub mod atmosphere_system_test;
pub mod bubble_buckey_balls;
pub mod clouds;
pub mod filtering_procedurals;
pub mod flappy_bird;
pub mod galaxy_of_universes;
pub mod geodesic_tiling;
pub mod heart;
pub mod loading_repeating_circles;
pub mod luminescence;
pub mod mandelbrot_smooth;
pub mod miracle_snowflakes;
pub mod morphing;
pub mod moving_square;
pub mod on_off_spikes;
pub mod phantom_star;
pub mod playing_marble;
pub mod protean_clouds;
pub mod raymarching_primitives;
pub mod seascape;
pub mod skyline;
pub mod soft_shadow_variation;
pub mod tileable_water_caustic;
pub mod tokyo;
pub mod two_tweets;
pub mod voxel_pac_man;

pub trait SampleCube: Copy {
  fn sample_cube(self, p: Vec3) -> Vec4;
}

#[derive(Copy, Clone)]
struct ConstantColor {
  color: Vec4,
}

impl SampleCube for ConstantColor {
  fn sample_cube(self, _: Vec3) -> Vec4 {
    self.color
  }
}

#[derive(Copy, Clone)]
struct RgbCube {
  alpha: f32,
  intensity: f32,
}

impl SampleCube for RgbCube {
  fn sample_cube(self, p: Vec3) -> Vec4 {
    (p.abs() * self.intensity).extend(self.alpha)
  }
}

pub struct ShaderInput {
  resolution: Vec3,
  time: f32,
  frag_coord: Vec2,
  mouse: Vec4,
}

pub struct ShaderResult {
  color: Vec4,
}

pub struct ShaderDefinition {
  name: &'static str,
}

macro_rules! match_index {
    ($e:expr; $($result:expr),* $(,)?) => ({
        let mut i = 0..;
        match $e { e => {
            $(if e == i.next().unwrap() { $result } else)*
            { unreachable!() }
        }}
    })
}

macro_rules! render_shader_macro {
    ($num_shaders:expr, $($shader_name:ident),* $(,)?) => {
        #[inline(always)]
        pub fn render_shader(shader_index: u32, shader_input: &ShaderInput, shader_output: &mut ShaderResult) {
            match_index!(shader_index; $(
                $shader_name::shader_fn(shader_input, shader_output),
            )*)
        }

        pub const SHADER_DEFINITIONS: [ShaderDefinition; $num_shaders] = [
            $(
                $shader_name::SHADER_DEFINITION,
            )*
        ];
    };
}

render_shader_macro!(
  28,
  two_tweets,
  heart,
  clouds,
  mandelbrot_smooth,
  protean_clouds,
  tileable_water_caustic,
  apollonian,
  phantom_star,
  seascape,
  playing_marble,
  a_lot_of_spheres,
  a_question_of_time,
  galaxy_of_universes,
  atmosphere_system_test,
  soft_shadow_variation,
  miracle_snowflakes,
  morphing,
  bubble_buckey_balls,
  raymarching_primitives,
  moving_square,
  skyline,
  filtering_procedurals,
  geodesic_tiling,
  flappy_bird,
  tokyo,
  on_off_spikes,
  luminescence,
  voxel_pac_man,
);

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
  render_shader(constants.shader_to_show, &shader_input, &mut shader_output);
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
