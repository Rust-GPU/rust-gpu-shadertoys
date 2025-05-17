//! Created by raldone01 :D

use core::f32::consts::PI;

use shared::*;
use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec3Swizzles, Vec4};

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{ShaderDefinition, ShaderInput, ShaderResult};

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
  name: "Loading Repeating Circles",
};

pub fn shader_fn(render_instruction: &ShaderInput, render_result: &mut ShaderResult) {
  let color = &mut render_result.color;
  let (resolution, time, frag_coord) = (
    render_instruction.resolution,
    render_instruction.time,
    render_instruction.frag_coord,
  );
  Inputs { resolution, time }.main_image(color, frag_coord)
}

pub struct Inputs {
  pub resolution: Vec3,
  pub time: f32,
}

fn circle_outline(uv: Vec2, center: Vec2, radius: f32, thickness: f32) -> f32 {
  // Compute distance from pixel to circle center.
  let dist = (uv - center).length();
  // Half thickness for symmetric band.
  let half_th = thickness * 0.5;
  // Return 1 if pixel is within the thickness band.
  return (dist - radius).abs().step(half_th);
}

/// The speed is in degrees per second.
fn rotating_discrete_circle(
  center: Vec2,
  radius: f32,
  time: f32,
  speed: f32,
  num_circles: i32,
  cirle_index: i32,
) -> Vec2 {
  // convert speed from degrees/sec to radians/sec
  let speed_rad = speed * PI / 180.0;
  // angle step between discrete circles
  let angle_step = 2.0 * PI / num_circles as f32;
  // base angle for this circle index
  let base_angle = angle_step * cirle_index as f32;
  // total rotation angle
  let angle = base_angle + time * speed_rad;
  // compute offset from center
  let offset = vec2(angle.cos(), angle.sin()) * radius;
  // return world‐space position
  center + offset
}

impl Inputs {
  pub fn main_image(&self, frag_color: &mut Vec4, frag_coord: Vec2) {
    // Get screen dimensions as Vec2.
    let screen_xy = self.resolution.xy();
    // Determine the shorter dimension of the screen.
    let shorter_dim = screen_xy.min_element();

    // Compute normalized pixel coordinates.
    // This maps the center of the screen to (0,0) and the shortest side to [-1,1].
    // Aspect ratio is preserved.
    let uv = (frag_coord - screen_xy * 0.5) / shorter_dim * 2.0;

    let aspect = screen_xy / shorter_dim;

    let mut combined_mask: f32 = 0.0;

    let center = vec2(0.0, 0.0);
    let bottom_middle = vec2(0.0, -aspect.y);
    let top_middle = vec2(0.0, aspect.y);
    let left_middle = vec2(-aspect.x, 0.0);
    let right_middle = vec2(aspect.x, 0.0);

    // Define circle radius and outline thickness.
    // These values are relative to the shorter screen dimension.
    let radius = 0.3;
    let thickness = 0.01;

    // Outline masks for center + four midpoints
    let m_center = circle_outline(uv, center, radius, thickness);
    combined_mask = combined_mask.max(m_center);
    let m_bot = circle_outline(uv, bottom_middle, radius, thickness);
    combined_mask = combined_mask.max(m_bot);
    let m_top = circle_outline(uv, top_middle, radius, thickness);
    combined_mask = combined_mask.max(m_top);
    let m_left = circle_outline(uv, left_middle, radius, thickness);
    combined_mask = combined_mask.max(m_left);
    let m_right = circle_outline(uv, right_middle, radius, thickness);
    combined_mask = combined_mask.max(m_right);

    // rotating circles
    let num_circles = 8;
    let speed = 2.0;
    for i in 0..num_circles {
      // Compute the position of the circle based on the angle and radius.
      let pos = rotating_discrete_circle(bottom_middle, 0.5, self.time, speed, num_circles, i);
      // Compute the outline mask for the current circle.
      let m = circle_outline(uv, pos, radius, thickness);
      // Combine masks using max to create a single mask.
      combined_mask = combined_mask.max(m);
    }

    // Mix white and black based on mask.
    // With current mask: outside band (mask=1) is black, inside band (mask=0) is white.
    // This produces a white outline on a black background.
    let color_rgb = mix(Vec3::ONE, Vec3::ZERO, combined_mask);

    // Output final pixel color with alpha = 1.0.
    *frag_color = color_rgb.extend(1.0);
  }
}
