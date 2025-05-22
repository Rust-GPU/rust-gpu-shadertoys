//! Created by raldone01 :D

use crate::shader_prelude::*;

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
  Inputs {
    resolution,
    time,
    frame: (time * 60.0) as i32,
    target_framerate: 60.0,
  }
  .main_image(color, frag_coord)
}

pub struct Inputs {
  pub resolution: Vec3,
  pub time: f32,
  pub frame: i32,
  pub target_framerate: f32,
}

const EPSILON: f32 = 1.0e-6;

fn calculate_H(aspect: Vec2, outer_circle_radius: f32, angle_between_circles: f32) -> f32 {
  let h = aspect.y * 2.0;
  let w = aspect.x * 2.0;

  // Calculate trigonometric values related to the arrangement of outer circles
  // alpha_rad is the angle between lines from the origin to centers of adjacent "middle_circles"
  // if they were arranged around the origin. The problem's geometry is a bit different,
  // but alpha_rad is used to find one specific outer circle's position.
  let alpha_rad = angle_between_circles;
  let S_alpha = f32::sin(alpha_rad);
  let C_alpha = f32::cos(alpha_rad);

  // Initialize max_H to a very small number (acting as negative infinity)
  // This will store the maximum valid H found among candidates.
  let mut max_H = -1.0e37;
  let mut found_valid_H = false;

  // X represents the 'middle_circle_radius' from the Python code, which is -h/2.0 - H.
  // We solve for X first, then find H = -X - h/2.0.
  // For H to be maximized, X must be minimized (i.e., most negative).

  // --- Candidate 1: Tangency to the Viewport's Bottom-Left Corner ---
  // This involves solving a quadratic equation for X: A_quad*X^2 + B_quad*X + C_term_quadratic = 0
  let A_quad = 2.0 * (1.0 - C_alpha);
  let B_quad = S_alpha * w + (1.0 - C_alpha) * h;
  let C_term_quadratic = (w * w + h * h) * 0.25 - outer_circle_radius * outer_circle_radius;

  let mut discriminant_val = B_quad * B_quad - 4.0 * A_quad * C_term_quadratic;

  if discriminant_val >= -EPSILON {
    // Allow for small negative due to precision
    discriminant_val = f32::max(0.0, discriminant_val); // Clamp to non-negative

    if f32::abs(A_quad) > EPSILON {
      // Avoid division by zero if A_quad is effectively zero
      // We need the solution for X that makes H positive, typically the more negative X.
      let X_corner = (-B_quad - f32::sqrt(discriminant_val)) / (2.0 * A_quad);

      // Validity conditions for corner tangency:
      // The outer circle's center (x_c, y_c) must be in the region "beyond" the bottom-left corner.
      // x_c = X_corner * S_alpha
      // y_c = X_corner * (1.0 - C_alpha)
      // Both S_alpha and (1.0-C_alpha) are non-negative for typical outer_circle_count.
      // X_corner is negative. So x_c and y_c will be <= 0.
      let x_cond_met = (X_corner * S_alpha <= -w / 2.0 + EPSILON);
      let y_cond_met = (X_corner * (1.0 - C_alpha) <= -h / 2.0 + EPSILON);

      if (x_cond_met && y_cond_met) {
        let H_candidate_corner = -X_corner - h / 2.0;
        // H must be non-negative (allow for small float errors)
        if (H_candidate_corner >= -EPSILON) {
          max_H = f32::max(max_H, H_candidate_corner);
          found_valid_H = true;
        }
      }
    }
    // else if A_quad is zero: This implies C_alpha=1 (e.g. outer_circle_count=1).
    // Then S_alpha=0, B_quad=0. Equation becomes C_term_quadratic = 0.
    // This means circle is at origin, tangency depends only on fixed params, not H.
    // This case is skipped, which is fine as H wouldn't be determinable by this candidate.
  }

  // --- Candidate 2: Tangency to the Viewport's Bottom Edge ---
  // y_c = -h/2.0 - outer_circle_radius
  // X*(1.0-C_alpha) = -h/2.0 - outer_circle_radius
  let term_1_minus_C_alpha = 1.0 - C_alpha;
  if (f32::abs(term_1_minus_C_alpha) > EPSILON) {
    // Avoid division by zero
    let X_bottom = (-h / 2.0 - outer_circle_radius) / term_1_minus_C_alpha;

    // Validity condition: x_c must be within the viewport's horizontal span.
    // x_c = X_bottom * S_alpha
    let x_c_check = X_bottom * S_alpha;
    if (x_c_check >= -w / 2.0 - EPSILON && x_c_check <= w / 2.0 + EPSILON) {
      let H_candidate_bottom = -X_bottom - h / 2.0;
      if (H_candidate_bottom >= -EPSILON) {
        max_H = f32::max(max_H, H_candidate_bottom);
        found_valid_H = true;
      }
    }
  }
  // else if (1.0-C_alpha) is zero: y_c = 0. Condition becomes 0 = -h/2 - outer_circle_radius.
  // Impossible for positive h, outer_circle_radius. Skipped.

  // --- Candidate 3: Tangency to the Viewport's Left Edge ---
  // x_c = -w/2.0 - outer_circle_radius
  // X*S_alpha = -w/2.0 - outer_circle_radius
  if (f32::abs(S_alpha) > EPSILON) {
    // Avoid division by zero
    let X_left = (-w / 2.0 - outer_circle_radius) / S_alpha;

    // Validity condition: y_c must be within the viewport's vertical span.
    // y_c = X_left * (1.0 - C_alpha)
    let y_c_check = X_left * (1.0 - C_alpha);
    if (y_c_check >= -h / 2.0 - EPSILON && y_c_check <= h / 2.0 + EPSILON) {
      let H_candidate_left = -X_left - h / 2.0;
      if (H_candidate_left >= -EPSILON) {
        max_H = f32::max(max_H, H_candidate_left);
        found_valid_H = true;
      }
    }
  }
  // else if S_alpha is zero: x_c = 0. Condition becomes 0 = -w/2 - outer_circle_radius.
  // Impossible for positive w, outer_circle_radius. Skipped.

  if (!found_valid_H) {
    // No valid H found (e.g., for degenerate inputs like outer_circle_count=1,
    // or if parameters lead to no physical solution).
    return -1.0; // Return sentinel value for error/no solution
  }

  // Ensure H is non-negative, clamping small negative results from precision errors to 0.
  return f32::max(0.0, max_H);
}

fn get_sausage_cap_extension_angle(half_stroke: f32, spine_radius: f32) -> f32 {
  // If R is zero or negative, the geometry is undefined or trivial.
  // Avoid division by zero.
  if spine_radius <= 0.0 {
    return 0.0;
  }
  let r = half_stroke.abs();

  let R_squared = spine_radius * spine_radius;
  let r_squared = r * r;

  let mut cos_alpha = 1.0 - r_squared / (2.0 * R_squared);

  // The argument to acos must be in the range [-1.0, 1.0].
  // If r is very large (e.g., r > 2*R), cos_alpha can be < -1.0.
  // clamp() ensures the value is within the valid domain for acos.
  // If r = 2R, cos_alpha = 1.0 - (4R^2)/(2R^2) = 1.0 - 2.0 = -1.0. acos(-1.0) = PI.
  // This corresponds to the small circle being large enough to intersect at (0, -R).
  // If r = 0, cos_alpha = 1.0. acos(1.0) = 0.0. (Small circle is a point at (0,R)).
  cos_alpha = f32::clamp(cos_alpha, -1.0, 1.0);

  let alpha_radians = f32::acos(cos_alpha);

  return alpha_radians;
}

/// SDF for a filled sausage shape (arc with thickness and rounded ends).
/// uv: Current pixel coordinate.
/// center_shape: Center of the arc for the sausage spine.
/// start_angle, end_angle: Define the arc of the spine (in radians). The arc is drawn CCW from start_angle.
///                         The length of the arc is `(end_angle - start_angle).rem_euclid(TWO_PI)`.
/// spine_radius: Radius of the arc spine.
/// opaque_percentage: Percentage of the sausage that is opaque, then it fades to transparent.
/// Returns: vec2(sdf_value, fade_intensity)
/// sdf_value: distance to surface (<0 inside, >0 outside)
/// fade_intensity: 0 (fully faded/transparent at the very start of tail) to 1 (fully opaque)
fn sdf_sausage_filled(
  uv: Vec2,
  center_shape: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  stroke: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> Vec2 {
  let half_stroke = stroke * 0.5;
  let p_local = uv - center_shape; // Point relative to arc center

  let mut effective_arc_length_angle = (end_angle - start_angle).rem_euclid(TWO_PI);

  if effective_arc_length_angle < EPSILON {
    if (start_angle - end_angle).abs() > EPSILON {
      effective_arc_length_angle = TWO_PI;
    }
  }

  let sdf_value: f32;

  if effective_arc_length_angle < EPSILON {
    // Case 1: Arc is effectively a point (a single circle)
    let arc_spine_point_local = vec2(start_angle.cos(), start_angle.sin()) * spine_radius;
    sdf_value = (p_local - arc_spine_point_local).length() - half_stroke;
  } else if (effective_arc_length_angle - TWO_PI).abs() < EPSILON {
    // Case 2: Arc is a full circle (annulus)
    sdf_value = (p_local.length() - spine_radius).abs() - half_stroke;
  } else {
    // Case 3: Arc is a partial arc (with rounded caps)
    let mid_angle_of_arc = start_angle + effective_arc_length_angle / 2.0;
    let rot_angle_for_symmetry = -mid_angle_of_arc;

    let cs_rot = rot_angle_for_symmetry.cos();
    let sn_rot = rot_angle_for_symmetry.sin();

    let p_sym_x = p_local.x * cs_rot - p_local.y * sn_rot;
    let p_sym_y = p_local.x * sn_rot + p_local.y * cs_rot;
    let p_sym = vec2(p_sym_x, p_sym_y);

    let half_arc_span_angle = effective_arc_length_angle / 2.0;
    let angle_p_sym = p_sym.y.atan2(p_sym.x); // Angle of p_sym in [-PI, PI]

    // Spine endpoints in the symmetric frame:
    // Start cap center: (R*cos(h_angle), -R*sin(h_angle))
    // End cap center:   (R*cos(h_angle),  R*sin(h_angle))
    let cap_spine_end_x_sym = spine_radius * half_arc_span_angle.cos();
    let cap_spine_end_y_abs_sym = spine_radius * half_arc_span_angle.sin();

    if angle_p_sym.abs() <= half_arc_span_angle + EPSILON {
      // Point is within or on the boundary of the angular "wedge" of the arc body.
      sdf_value = (p_sym.length() - spine_radius).abs() - half_stroke;
    } else {
      // Point is outside the wedge, closer to one of the rounded caps.
      let chosen_cap_center_sym: Vec2;
      if angle_p_sym > half_arc_span_angle {
        // Closer to the "end" cap (positive Y side in symm. frame)
        chosen_cap_center_sym = vec2(cap_spine_end_x_sym, cap_spine_end_y_abs_sym);
      } else {
        // angle_p_sym < -half_arc_span_angle: Closer to the "start" cap (negative Y side in symm. frame)
        chosen_cap_center_sym = vec2(cap_spine_end_x_sym, -cap_spine_end_y_abs_sym);
      }
      sdf_value = (p_sym - chosen_cap_center_sym).length() - half_stroke;
    }
  }

  // Fade out the sausage
  let fade_intensity: f32;

  let cap_extension_angle = get_sausage_cap_extension_angle(half_stroke, spine_radius);
  let fade_start_angle = start_angle - cap_extension_angle;
  let fade_end_angle = end_angle + cap_extension_angle;
  fade_intensity = circular_fade_out(
    uv,
    center_shape,
    fade_start_angle,
    fade_end_angle,
    fade_center_angle,
    opaque_percentage,
  );

  vec2(sdf_value, fade_intensity)
}

/// Computes a circular angular fade-out based on direction from center.
/// Returns 1.0 in the opaque region, fades to 0.0 outside it.
fn circular_fade_out(
  uv: Vec2,
  center: Vec2,
  start_angle: f32,
  end_angle: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> f32 {
  let dir = (uv - center).normalize();
  let mut angle = dir.y.atan2(dir.x);
  if angle < 0.0 {
    angle += TWO_PI;
  }

  let arc_start = start_angle.rem_euclid(TWO_PI);
  let arc_end = end_angle.rem_euclid(TWO_PI);
  let mut span = arc_end - arc_start;
  if span < 0.0 {
    span += TWO_PI;
  }

  if span < 0.00001 {
    return if opaque_percentage >= 0.9999 {
      1.0
    } else {
      0.0
    };
  }

  let mut rel_angle = angle - arc_start;
  if rel_angle < 0.0 {
    rel_angle += TWO_PI;
  }

  if rel_angle > span + 0.0001 || rel_angle < -0.0001 {
    return 0.0;
  }
  rel_angle = rel_angle.clamp(0.0, span);

  let norm_angle = rel_angle / span;

  let fade_center = fade_center_angle.rem_euclid(TWO_PI);
  let mut center_rel = fade_center - arc_start;
  if center_rel < 0.0 {
    center_rel += TWO_PI;
  }
  center_rel = center_rel.clamp(0.0, span);
  let norm_center = center_rel / span;

  let opaque = opaque_percentage.clamp(0.0, 1.0);
  let half_width = opaque / 2.0;
  let op_start = norm_center - half_width;
  let op_end = norm_center + half_width;

  if norm_angle >= op_start && norm_angle <= op_end {
    return 1.0;
  } else if norm_angle < op_start {
    if op_start <= 0.00001 {
      return 0.0;
    } else {
      return smoothstep(0.0, op_start, norm_angle);
    }
  } else {
    if op_end >= 1.0 - 0.00001 {
      return 0.0;
    } else {
      return 1.0 - smoothstep(op_end, 1.0, norm_angle);
    }
  }
}

fn sdf_sausage_outline(
  uv: Vec2,
  center_shape: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  inner_radius: f32,
  outer_radius: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> Vec2 {
  let m_inner = sdf_sausage_filled(
    uv,
    center_shape,
    start_angle,
    end_angle,
    spine_radius,
    inner_radius * 2.0,
    fade_center_angle,
    opaque_percentage,
  );
  let m_outer = sdf_sausage_filled(
    uv,
    center_shape,
    start_angle,
    end_angle,
    spine_radius,
    outer_radius * 2.0,
    fade_center_angle,
    opaque_percentage,
  );
  // remove m_inner from m_outer
  let sdf_value = m_outer.x.max(-m_inner.x);
  let fade_intensity = m_outer.y; // or m_inner.y, as they should be the same
  vec2(sdf_value, fade_intensity)
}

fn sdf_circle_outline(uv: Vec2, center: Vec2, radius: f32, stroke: f32) -> f32 {
  // Compute distance from pixel to circle center.
  let dist = (uv - center).length();
  // Half stroke for symmetric band.
  let half_th = stroke * 0.5;
  // Return 1 if pixel is within the stroke band.
  return (dist - radius).abs().step(half_th);
}

fn sdf_circle_outline_2(uv: Vec2, center: Vec2, inner_radius: f32, outer_radius: f32) -> f32 {
  // Compute distance from pixel to circle center.
  let dist = (uv - center).length();
  // Return 1 if pixel is within the stroke band.
  return (dist - inner_radius)
    .abs()
    .step(outer_radius - inner_radius);
}

/// Returns a value along an exponential curve shaped by `c`.
/// `t` should be in [0.0, 1.0].
pub fn exp_time(t: f32, c: f32) -> f32 {
  let c = c * 10.0;

  if c.abs() < EPSILON {
    t
  } else {
    let numerator = (c * t).exp() - 1.0;
    let denominator = c.exp() - 1.0;
    numerator / denominator
  }
}

struct RotatingCircleResult {
  position: Vec2,
  angle: f32,
}

/// t goes from 0 to 1
fn rotating_discrete_circle(
  center: Vec2,
  radius: f32,
  start_angle: f32,
  num_circles: i32,
  cirle_index: i32,
) -> RotatingCircleResult {
  // angle step between discrete circles
  let angle_step = 2.0 * PI / num_circles as f32;
  // base angle for this circle index
  let base_angle = angle_step * cirle_index as f32;
  // total rotation angle
  let angle = base_angle + start_angle;
  // compute offset from center
  let offset = vec2(angle.cos(), angle.sin()) * radius;
  // return world‐space position
  RotatingCircleResult {
    position: center + offset,
    angle: angle,
  }
}

/// General purpose function to remap a time segment to 0-1.
/// parent_t: The main animation time, expected to be 0-1.
/// start_time: The point in parent_t (0-1) where this sub-animation should begin.
/// end_time: The point in parent_t (0-1) where this sub-animation should end.
/// Returns: 0.0 before start_time, 1.0 after end_time, and a 0-1 ramp between them.
fn remap_time(parent_t: f32, start_time: f32, end_time: f32) -> f32 {
  if start_time >= end_time {
    // If start and end are the same, or invalid order:
    // Option 1: return 0 if parent_t < startTime, 1 if parent_t >= endTime (instant step)
    // Option 2: return 0 always (safer for division by zero avoidance)
    return if parent_t >= start_time { 1.0 } else { 0.0 }; // Option 1 (step)
                                                           // return 0.0; // Option 2 (safer)
  }
  let duration = end_time - start_time;
  return f32::clamp((parent_t - start_time) / duration, 0.0, 1.0);
}

/// Alpha compositing using "over" operator
fn composite_layers<const N: usize>(overlay_colors: &[Vec4; N]) -> Vec4 {
  let mut result = Vec4::ZERO;
  for i in (0..overlay_colors.len()).rev() {
    let color = overlay_colors[i];
    let alpha = color.w;
    if alpha > 0.0 {
      result = result + (color * alpha * (1.0 - result.w));
      result.w += alpha * (1.0 - result.w);
    }
  }
  return result;
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
    let aa_width = 2.0 / screen_xy.max_element();

    let aspect = screen_xy / shorter_dim;

    let mut black_alpha: f32 = 0.0;
    let mut debug_red_alpha: f32 = 0.0;
    let mut debug_blue_alpha: f32 = 0.0;

    let center = vec2(0.0, 0.0);
    let bottom_middle = vec2(0.0, -aspect.y);
    let top_middle = vec2(0.0, aspect.y);
    let left_middle = vec2(-aspect.x, 0.0);
    let right_middle = vec2(aspect.x, 0.0);

    let start_radius = (bottom_middle - center).length();
    let target_radius = 0.2;
    let target_stroke = 0.05;
    let m_target = sdf_circle_outline(uv, center, target_radius, target_stroke);
    //combined_mask = combined_mask.max(m_target);
    let period = 4.0; // seconds
    let t_master = (self.time / period).fract();
    //let t_master = 0.95;
    //let t_master = 0.8;
    //let t_master = 0.6;
    //let t_master = 0.5;
    let t_middle = exp_time(t_master, -0.4);
    let t_rotation = exp_time(t_master, 0.6);
    let t_trail_delayed = remap_time(t_master, 0.4, 1.0);
    let t_trail = exp_time(t_trail_delayed, 0.4);
    let t_assist_circle_delayed = remap_time(t_master, 0.5, 0.9);
    let t_assist_circle = exp_time(t_assist_circle_delayed, 0.4);
    //let t_exp = 1.0 - (-5.0 * t).exp();

    // rotating circles
    let num_circles = 12;
    let angle_between_circles = 2.0 * PI / num_circles as f32;
    let mut H = 0.0;
    for i in 0..num_circles {
      let circle_angle = angle_between_circles * i as f32;
      let H_candidate = calculate_H(aspect, target_radius + target_stroke / 2.0, circle_angle);
      if H_candidate > H {
        H = H_candidate;
      }
    }
    H = H * 1.6;
    // move from bottom_middle to center
    let middle_circle_start_radius = start_radius + H;
    let middle_circle_radius = mix(middle_circle_start_radius, target_radius, t_middle);
    let middle_circle_start_position = bottom_middle - Vec2::new(0.0, H);
    let mut middle_circle_position = mix(middle_circle_start_position, center, t_middle);
    let middle_circle_moved_distance =
      (middle_circle_start_position - middle_circle_position).length();
    let outer_circle_outer_radius = (middle_circle_start_radius + target_radius)
      - (middle_circle_radius + middle_circle_moved_distance);
    let trail_angular_extent = mix(0.0, angle_between_circles, t_trail);
    let outer_circle_fade = mix(1.0, 0.4, t_trail);

    // y adjust to follow the middle circle
    middle_circle_position.y -=
      (middle_circle_position.y + middle_circle_radius) * (1.0 - t_master);

    let m_start_circle = sdf_circle_outline(uv, Vec2::ZERO, target_radius, target_stroke);
    debug_red_alpha = debug_red_alpha.max(m_start_circle);
    let m_middle_circle_path = sdf_circle_outline(
      uv,
      middle_circle_start_position,
      middle_circle_start_radius,
      target_stroke / 2.0,
    );
    debug_red_alpha = debug_red_alpha.max(m_middle_circle_path);

    for i in 0..num_circles {
      // Compute the position of the circle based on the angle and radius.
      let outer_discrete_circle = rotating_discrete_circle(
        middle_circle_position,
        middle_circle_radius,
        -t_rotation * TWO_PI * 5.0,
        num_circles,
        i,
      );

      let outer_circle_inner_radius = (outer_circle_outer_radius - target_stroke / 2.0).max(0.0);
      let outer_circle_outer_radius = outer_circle_outer_radius + target_stroke / 2.0;
      let m = sdf_sausage_outline(
        uv,
        middle_circle_position,
        outer_discrete_circle.angle - trail_angular_extent / 2.0,
        outer_discrete_circle.angle + trail_angular_extent / 2.0,
        middle_circle_radius,
        outer_circle_inner_radius,
        outer_circle_outer_radius,
        outer_discrete_circle.angle,
        outer_circle_fade,
      );
      black_alpha =
        black_alpha.max((1.0 - smoothstep(0.0, aa_width, m.x)) * (m.y + t_assist_circle).min(1.0));
      // * (m.y + 6.0 / 256.0));

      let m = sdf_circle_outline(
        uv,
        middle_circle_position,
        middle_circle_radius,
        outer_circle_outer_radius * 2.0,
      );
      black_alpha = black_alpha.max((smoothstep(0.0, aa_width, m)) * t_assist_circle);
    }

    let color_background = Vec4::ONE;
    let color_black = Vec4::new(0.0, 0.0, 0.0, black_alpha);
    //debug_red_alpha = 0.0;
    let color_red = Vec4::new(1.0, 0.0, 0.0, debug_red_alpha * 0.5);
    let color_blue = Vec4::new(0.0, 0.0, 1.0, debug_blue_alpha * 0.5);

    let color_rgb = composite_layers(&[color_background, color_black, color_red, color_blue]);

    // Output final pixel color with alpha = 1.0.
    *frag_color = color_rgb;
  }
}
