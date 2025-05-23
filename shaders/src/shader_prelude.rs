pub use core::f32::consts::{FRAC_1_PI, FRAC_PI_2, PI};
pub const TWO_PI: f32 = 2.0 * PI;
pub const SQRT3: f32 = 1.7320508075688772;

pub use shared::*;
pub use spirv_std::{
  arch::Derivative,
  glam::{
    mat2, mat3, vec2, vec3, vec4, Mat2, Mat3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4,
    Vec4Swizzles,
  },
  spirv,
};

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
pub use spirv_std::num_traits::Float;

pub trait SampleCube: Copy {
  fn sample_cube(self, p: Vec3) -> Vec4;
}

#[derive(Copy, Clone)]
pub struct ConstantColor {
  pub color: Vec4,
}

impl SampleCube for ConstantColor {
  fn sample_cube(self, _: Vec3) -> Vec4 {
    self.color
  }
}

#[derive(Copy, Clone)]
pub struct RgbCube {
  pub alpha: f32,
  pub intensity: f32,
}

impl SampleCube for RgbCube {
  fn sample_cube(self, p: Vec3) -> Vec4 {
    (p.abs() * self.intensity).extend(self.alpha)
  }
}

pub struct ShaderInput {
  pub resolution: Vec3,
  pub time: f32,
  pub frag_coord: Vec2,
  /// https://www.shadertoy.com/view/Mss3zH
  pub mouse: Vec4,
}

pub struct ShaderResult {
  pub color: Vec4,
}

pub struct ShaderDefinition {
  pub name: &'static str,
}
