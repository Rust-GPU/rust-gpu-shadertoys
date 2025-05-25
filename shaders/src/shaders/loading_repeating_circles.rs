//! Created by raldone01 :D
//! Special thanks to Thehanna on MathSE
use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
  name: "Loading Repeating Circles",
};

pub fn shader_fn(render_instruction: &ShaderInput, render_result: &mut ShaderResult) {
  let color = &mut render_result.color;
  let (resolution, time, frag_coord, mouse) = (
    render_instruction.resolution,
    render_instruction.time,
    render_instruction.frag_coord,
    render_instruction.mouse,
  );
  Inputs {
    resolution,
    time,
    frame: (time * 60.0) as i32,
    mouse,
  }
  .main_image(color, frag_coord)
}

pub struct Inputs {
  pub resolution: Vec3,
  pub time: f32,
  pub frame: i32,
  pub mouse: Vec4,
}

/// Epsilon used for floating-point comparisons.
const EPSILON: f32 = 1.0e-6;
/// A small constant gap between stacked progress bars.
const PROGRESS_BAR_GAP: f32 = 0.01;

/// An SDF value that can be negative (inside the shape) or positive (outside the shape).
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct SDFValue(f32);
impl SDFValue {
  pub const FAR_OUTSIDE: SDFValue = SDFValue(f32::MAX);
  pub const FAR_INSIDE: SDFValue = SDFValue(f32::MIN);
  pub const SURFACE: SDFValue = SDFValue(0.0);

  /// Creates a new SDFValue.
  #[must_use]
  pub fn new(value: f32) -> Self {
    SDFValue(value)
  }

  /// Returns the raw f32 distance.
  #[must_use]
  pub fn value(self) -> f32 {
    self.0
  }

  /// Returns `true` if the SDF value is inside the shape (negative).
  #[must_use]
  pub fn is_inside(self) -> bool {
    self.0 < 0.0
  }

  /// Returns `true` if the SDF value is outside the shape (positive).
  #[must_use]
  pub fn is_outside(self) -> bool {
    self.0 > 0.0
  }

  /// Converts the SDF value to an alpha value for anti-aliasing.
  /// Alpha is 1.0 deep inside, 0.0 deep outside, and smooth in between.
  /// The transition happens from `aa_width` (alpha 0) to `-aa_width` (alpha 1).
  #[must_use]
  pub fn to_alpha(self, aa_width: f32) -> f32 {
    if aa_width <= EPSILON {
      return self.is_inside() as i32 as f32;
    }

    return smoothstep(aa_width, -aa_width, self.0);
  }

  /// Insets the shape by a given thickness.
  /// This is done by shrinking the shape and then calculating the difference.
  #[must_use]
  pub fn inset(self, amount: f32) -> Self {
    Self(self.0.max(-(self.0 + amount)))
  }

  /// Expands (if amount > 0) or shrinks (if amount < 0) the shape.
  /// This is equivalent to subtracting from the distance value.
  #[must_use]
  pub fn offset(&self, amount: f32) -> Self {
    Self(self.0 - amount)
  }

  /// Takes the absolute value of the SDF, effectively creating an infinitely thin shell
  /// on the surface of the original shape.
  #[must_use]
  pub fn shell(self) -> Self {
    Self(self.0.abs())
  }

  /// Creates an outline (hollow shape) from the SDF.
  #[must_use]
  pub fn to_outline(self, thickness: f32) -> Self {
    Self(self.0.abs() - thickness)
  }

  /// Difference operation (self - other). Result is inside if inside self AND outside other.
  /// Equivalent to Intersection(self, Invert(other)).
  #[must_use]
  pub fn difference(self, other: Self) -> Self {
    Self(self.0.max(-other.0))
  }

  /// Union operation (self U other). Result is inside if inside self OR inside other.
  #[must_use]
  pub fn union(self, other: Self) -> Self {
    Self(self.0.min(other.0))
  }

  /// Intersection operation (self ∩ other). Result is inside if inside self AND inside other.
  #[must_use]
  pub fn intersection(self, other: Self) -> Self {
    Self(self.0.max(other.0))
  }

  /// Inverts the SDF (inside becomes outside and vice-versa).
  #[must_use]
  pub fn invert(self) -> Self {
    Self(-self.0)
  }
}

/// Calculates the distance from the origin to the center of the initial main circle so that
/// at time `0`, only one border circle is visible, with the others just touching the sides/bottom of the viewport.
#[must_use]
fn calculate_initial_distance_for_main_circle_center(
  aspect: Vec2,
  border_circle_radius: f32,
  angle_between_circles: f32,
) -> Option<f32> {
  // Height and width of the viewport in normalized coordinates.
  let h_half = aspect.y;
  let h = h_half * 2.0;
  let w_half = aspect.x;
  let w = w_half * 2.0;

  // Calculate the sin/cos values for the border circle.
  let s_alpha = angle_between_circles.sin();
  let c_alpha = angle_between_circles.cos();
  let term_1_minus_c_alpha = 1.0 - c_alpha;

  let mut max_distance = f32::NEG_INFINITY;

  // Candidate 1: Corner Tangency to the left viewport corner.
  // x = MCR = -h/2.0 - H
  // (x*s_alpha + w/2)^2 + (x*(1-c_alpha) + h/2)^2 = border_circle_radius^2
  // a_quad*x^2 + b_quad*x + c_quad_term = 0
  let a_quad = 2.0 * term_1_minus_c_alpha;
  let b_quad = s_alpha * w + term_1_minus_c_alpha * h;
  let c_term_quadratic = (w * w + h * h) * 0.25 - border_circle_radius * border_circle_radius;

  let mut discriminant_val = b_quad * b_quad - 4.0 * a_quad * c_term_quadratic;

  if discriminant_val >= -EPSILON {
    // Allow for small negative due to precision
    discriminant_val = discriminant_val.max(0.0);

    if a_quad.abs() > EPSILON {
      // Minimize x to find maximum distance
      // Avoid division by zero if a_quad is effectively zero
      let x_corner = (-b_quad - discriminant_val.sqrt()) / (2.0 * a_quad);

      // Check validity conditions:
      // The outer circle's center (x_c, y_c) must be in the region "beyond" the bottom-left corner.
      // x_c = x_corner * s_alpha
      // y_c = x_corner * (1.0 - c_alpha)
      let x_cond_met = x_corner * s_alpha <= -w_half + EPSILON;
      let y_cond_met = x_corner * term_1_minus_c_alpha <= -h_half + EPSILON;

      if x_cond_met && y_cond_met {
        let distance_candidate_corner = -x_corner;
        // H must be non-negative (allow for small float errors)
        if distance_candidate_corner >= -EPSILON {
          max_distance = max_distance.max(distance_candidate_corner);
        }
      }
    }
  }

  // Candidate 2: Bottom Edge Tangency
  // y_c = -h/2.0 - border_circle_radius
  // x*(1.0-c_alpha) = -h/2.0 - border_circle_radius
  if term_1_minus_c_alpha.abs() > EPSILON {
    // Avoid division by zero
    let x_bottom = (-h_half - border_circle_radius) / term_1_minus_c_alpha;

    // Validity condition: x_c must be within the viewport's horizontal span.
    // x_c = x_bottom * s_alpha
    let x_c_check = x_bottom * s_alpha;
    if x_c_check >= -w_half - EPSILON && x_c_check <= w_half + EPSILON {
      let distance_candidate_bottom = -x_bottom;
      if distance_candidate_bottom >= -EPSILON {
        max_distance = max_distance.max(distance_candidate_bottom);
      }
    }
  }

  // Candidate 3: Left Edge Tangency
  // x_c = -w/2.0 - border_circle_radius
  // x*s_alpha = -w/2.0 - border_circle_radius
  if s_alpha.abs() > EPSILON {
    // Avoid division by zero
    let x_left = (-w_half - border_circle_radius) / s_alpha;

    // Validity condition: y_c must be within the viewport's vertical span.
    // y_c = x_left * (1.0 - c_alpha)
    let y_c_check = x_left * term_1_minus_c_alpha;
    if y_c_check >= -h_half - EPSILON && y_c_check <= h_half + EPSILON {
      let distance_candidate_left = -x_left;
      if distance_candidate_left >= -EPSILON {
        max_distance = max_distance.max(distance_candidate_left);
      }
    }
  }

  if max_distance == f32::NEG_INFINITY {
    return None;
  }

  // Ensure positivity :D.
  return Some(max_distance.max(0.0));
}

/// Given an arc radius and its half-stroke width,
/// this function computes the angle that the arc extends beyond its endpoints because of the stroke width.
#[must_use]
fn arc_cap_extension_angle(arc_radius: f32, half_stroke: f32) -> f32 {
  // Avoid division by zero.
  if arc_radius <= 0.0 {
    return 0.0;
  }
  let r = half_stroke;

  let big_r_squared = arc_radius * arc_radius;
  let r_squared = r * r;

  let mut cos_alpha = 1.0 - r_squared / (2.0 * big_r_squared);

  // The argument to acos must be in the range [-1.0, 1.0].
  // If r is very large (e.g., r > 2*R), cos_alpha can be < -1.0.
  // clamp() ensures the value is within the valid domain for acos.
  cos_alpha = cos_alpha.clamp(-1.0, 1.0);

  let alpha_radians = cos_alpha.acos();
  return alpha_radians;
}

/// SDF for an arc with rounded ends (sausage shape).
///
/// * `uv`: Current pixel coordinate.
/// * `start_angle`, `end_angle`: Defines the arc of the spine (in radians).
///                               The arc is drawn CCW from `start_angle`.
/// * `spine_radius`: Radius of the arc spine.
/// * `stroke`: Width of the arc body.
///
/// Returns the [`SDFValue`] for the arc.
#[must_use]
fn sdf_arc_filled(
  uv: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  stroke: f32,
) -> SDFValue {
  let half_stroke = stroke * 0.5;

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
    sdf_value = (uv - arc_spine_point_local).length() - half_stroke;
  } else if (effective_arc_length_angle - TWO_PI).abs() < EPSILON {
    // Case 2: Arc is a full circle (annulus)
    sdf_value = (uv.length() - spine_radius).abs() - half_stroke;
  } else {
    // Case 3: Arc is a partial arc (with rounded caps)
    let mid_angle_of_arc = start_angle + effective_arc_length_angle / 2.0;
    let rot_angle_for_symmetry = -mid_angle_of_arc;

    let cs_rot = rot_angle_for_symmetry.cos();
    let sn_rot = rot_angle_for_symmetry.sin();

    let p_sym_x = uv.x * cs_rot - uv.y * sn_rot;
    let p_sym_y = uv.x * sn_rot + uv.y * cs_rot;
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

  SDFValue::new(sdf_value)
}

/// Computes a circular angular fade-out based on direction from center.
///
/// * `uv`: Current pixel coordinate.
/// * `start_angle`, `end_angle`: Defines the arc of the spine (in radians).
///                               The arc is drawn CCW from `start_angle`.
/// * `spine_radius`: The radius of the arc spine.
/// * `stroke`: The width of the arc.
/// * `fade_center_angle`: The angle at which the fade starts.
///                        The fade will extend symmetrically around this angle.
/// * `opaque_percentage`: Percentage of the arc that is opaque.
///                        The fade starts at the edges of the opaque region.
/// * `fade_intensity`: `0` (fully faded/transparent) to `1` (fully opaque)
///
/// Returns `1` in the opaque region, fades to `0` outside it.
#[must_use]
fn arc_fade_out(
  uv: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  stroke: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> f32 {
  let half_stroke = stroke * 0.5;
  let cap_extension_angle = arc_cap_extension_angle(spine_radius, half_stroke);
  let fade_start_angle = start_angle - cap_extension_angle;
  let fade_end_angle = end_angle + cap_extension_angle;

  let dir = uv.normalize();
  let mut angle = dir.y.atan2(dir.x);
  if angle < 0.0 {
    angle += TWO_PI;
  }

  let arc_start = fade_start_angle.rem_euclid(TWO_PI);
  let arc_end = fade_end_angle.rem_euclid(TWO_PI);
  let mut effective_arc_length_angle = arc_end - arc_start;
  if effective_arc_length_angle < EPSILON {
    if (fade_start_angle - fade_end_angle).abs() > EPSILON {
      effective_arc_length_angle = TWO_PI;
    }
  }

  let mut rel_angle = angle - arc_start;
  if rel_angle < 0.0 {
    rel_angle += TWO_PI;
  }

  if rel_angle > effective_arc_length_angle + EPSILON || rel_angle < EPSILON {
    return 0.0;
  }
  rel_angle = rel_angle.clamp(0.0, effective_arc_length_angle);

  if opaque_percentage >= 1.0 - EPSILON {
    // If the opaque percentage is effectively 100%, we return 1.0.
    return 1.0;
  }

  let norm_angle = rel_angle / effective_arc_length_angle;

  let fade_center = fade_center_angle.rem_euclid(TWO_PI);
  let mut center_rel = fade_center - arc_start;
  if center_rel < 0.0 {
    center_rel += TWO_PI;
  }
  center_rel = center_rel.clamp(0.0, effective_arc_length_angle);
  let norm_center = center_rel / effective_arc_length_angle;

  let opaque = opaque_percentage.clamp(0.0, 1.0);
  let half_width = opaque / 2.0;
  let op_start = norm_center - half_width;
  let op_end = norm_center + half_width;

  if norm_angle >= op_start && norm_angle <= op_end {
    return 1.0;
  } else if norm_angle < op_start {
    if op_start <= EPSILON {
      return 0.0;
    } else {
      return smoothstep(0.0, op_start, norm_angle);
    }
  } else {
    if op_end >= 1.0 - EPSILON {
      return 0.0;
    } else {
      return 1.0 - smoothstep(op_end, 1.0, norm_angle);
    }
  }
}

/// SDF for an arc outline with rounded ends.
///
/// * `uv`: Current pixel coordinate.
/// * `start_angle`, `end_angle`: Defines the arc of the spine (in radians).
///                               The arc is drawn CCW from `start_angle`.
/// * `spine_radius`: Radius of the arc spine.
/// * `inner_radius`: Inner radius of the arc outline.
/// * `outer_radius`: Outer radius of the arc outline.
/// * `fade_center_angle`: The angle at which the fade starts.
///                        The fade will extend symmetrically around this angle.
/// * `opaque_percentage`: Percentage of the arc that is opaque.
///                        The fade starts at the edges of the opaque region.
///
/// Returns a tuple with the first component being the [`SDFValue`] for the arc outline,
/// and the second component being the fade intensity.
#[must_use]
fn sdf_arc_outline(
  uv: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  inner_radius: f32,
  outer_radius: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> (SDFValue, f32) {
  let mut sdf_value = sdf_arc_filled(uv, start_angle, end_angle, spine_radius, outer_radius * 2.0);
  if inner_radius > EPSILON {
    sdf_value = sdf_value.inset(outer_radius - inner_radius);
  }
  let fade_intensity = arc_fade_out(
    uv,
    start_angle,
    end_angle,
    spine_radius,
    outer_radius * 2.0,
    fade_center_angle,
    opaque_percentage,
  );
  (sdf_value, fade_intensity)
}

/// SDF for a filled circle.
///
/// * `uv`: The coordinates relative to the center of the circle.
/// * `radius`: The radius of the circle.
///
/// Returns the [`SDFValue`] for the circle.
#[must_use]
fn sdf_circle_filled(uv: Vec2, radius: f32) -> SDFValue {
  let d = uv.length() - radius;
  let outside_distance = d.max(0.0);
  let inside_distance = d.min(0.0);
  SDFValue::new(outside_distance + inside_distance)
}

/// Returns a value along an exponential curve shaped by `c`.
///
/// `c == 0` returns `t` (linear).
///
/// * `t` should be in `0..1`.
/// * `c` is usually in `-2..2`.
#[must_use]
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

/// Returns the derivative of the exponential function.
#[must_use]
pub fn exp_time_derivative(t: f32, c: f32) -> f32 {
  let c = c * 10.0;

  if c.abs() < EPSILON {
    return 1.0;
  }

  let numerator = c * (c * t).exp();
  let denominator = c.exp() - 1.0;
  numerator / denominator
}

#[must_use]
pub fn offset_loop_time(t: f32, offset: f32) -> f32 {
  // Apply offset
  let offset_t = t + offset;
  // Wrap around to [0, 1]
  return offset_t.rem_euclid(1.0);
}

pub struct RotatingCircleResult {
  position: Vec2,
  angle: f32,
}

#[must_use]
pub fn rotating_discrete_circle(
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

/// General purpose function to remap a time segment to `0..1`.
///
/// * `parent_t`: The main animation time, expected to be `0..1`.
/// * `start_time`: The point in parent_t (`0..1`) where this sub-animation should begin.
/// * `end_time`: The point in parent_t (`0..1`) where this sub-animation should end.
///
/// Returns: `0` before `start_time`, `1` after `end_time`, and a `0..1` ramp between them.
#[must_use]
pub fn remap_time(parent_t: f32, start_time: f32, end_time: f32) -> f32 {
  if start_time >= end_time {
    // If start and end are the same, or invalid order we do an instant step.
    return if parent_t >= start_time { 1.0 } else { 0.0 };
  }
  let duration = end_time - start_time;
  return ((parent_t - start_time) / duration).clamp(0.0, 1.0);
}

#[derive(Copy, Clone)]
pub struct Rectangle {
  pub position: Vec2,
  pub half_dimensions: Vec2,
}
impl Rectangle {
  /// Creates a new rectangle centred on the given position and with the given half-dimensions.
  #[must_use]
  pub fn new(position: Vec2, half_dimensions: Vec2) -> Self {
    Rectangle {
      position,
      half_dimensions,
    }
  }

  #[must_use]
  pub fn is_inside(&self, uv: Vec2) -> bool {
    let relative_uv = uv - self.position;
    relative_uv.x.abs() <= self.half_dimensions.x && relative_uv.y.abs() <= self.half_dimensions.y
  }

  /// Returns the SDF value for this rectangle at the given UV coordinates.
  #[must_use]
  pub fn sdf_box_filled(&self, uv: Vec2) -> SDFValue {
    sdf_box_filled(uv - self.position, self.half_dimensions)
  }
}

/// SDF for a filled box.
///
/// * `uv`: The coordinates relative to the center of the box.
/// * `positioning`: How the box is positioned relative to `uv`.
/// * `half_dimensions`: half-width and half-height of the box.
///
/// Returns the [`SDFValue`] for the box.
#[must_use]
fn sdf_box_filled(uv: Vec2, half_dimensions: Vec2) -> SDFValue {
  let d = uv.abs() - half_dimensions;
  let outside_distance = d.max(Vec2::ZERO).length();
  let inside_distance = d.x.max(d.y).min(0.0);

  SDFValue::new(outside_distance + inside_distance)
}

/// SDF for a progress bar rectangle.
/// Bars are stacked downwards from the top of the screen.
///
/// * `uv`: coordinates relative to the center of the screen.
/// * `aspect`: half-dimensions of the screen (half-width, half-height).
/// * `stroke`: This parameter will define the height of the time bar.
/// * `progress`: progress of the bar, from `0.0` (empty) to `1.0` (full).
/// * `index`: vertical stacking index of the bar. Index 0 is the top-most bar.
///
/// Returns the [`SDFValue`] for the progress bar.
#[must_use]
fn sdf_progress_bar(uv: Vec2, aspect: Vec2, stroke: f32, progress: f32, index: u32) -> SDFValue {
  let bar_height = stroke;
  // The bar spans the full width of the viewport.
  let bar_full_potential_width = aspect.x * 2.0;

  // Calculate the vertical position of the current bar.
  // `index = 0` is the top-most bar.
  // `aspect.y` is the Y-coordinate of the top edge of the screen.
  // Each subsequent bar (`index > 0`) is placed below the previous one.
  let bar_top_edge_y = aspect.y - (index as f32) * (bar_height + PROGRESS_BAR_GAP);
  let bar_vertical_center_y = bar_top_edge_y - bar_height / 2.0;

  // Calculate the current width of the filled portion of the bar.
  let current_filled_width = bar_full_potential_width * progress;

  // The filled portion of the bar starts from the left screen edge (`-aspect.x`).
  // Calculate the X-coordinate of the center of this filled portion.
  let filled_portion_horizontal_center_x = -aspect.x + current_filled_width / 2.0;

  // Define the center and half-dimensions of the filled rectangle.
  let fill_rect_center_pos = vec2(filled_portion_horizontal_center_x, bar_vertical_center_y);
  let fill_rect_half_dims = vec2(current_filled_width / 2.0, bar_height / 2.0);

  // --- SDF and Anti-aliasing ---
  // Transform current `uv` to be relative to the center of the filled rectangle.
  let uv_relative_to_fill_rect_center = uv - fill_rect_center_pos;

  // Calculate the signed distance to the boundary of the filled rectangle.
  // `sd_box` returns < 0 inside, 0 on boundary, > 0 outside.
  let distance_to_fill_boundary =
    sdf_box_filled(uv_relative_to_fill_rect_center, fill_rect_half_dims);

  distance_to_fill_boundary
}

/// Draws a filled progress bar rectangle that fills in discrete steps,
/// with new steps fading in.
/// Bars are stacked downwards from the top of the screen.
///
/// * `uv`: coordinates relative to the center of the screen.
/// * `aspect`: half-dimensions of the screen (half-width, half-height).
/// * `stroke`: This parameter will define the height of the time bar.
/// * `progress`: overall progress of the bar, from 0.0 (empty) to 1.0 (full).
/// * `index`: vertical stacking index of the bar. Index 0 is the top-most bar.
/// * `steps`: the number of discrete steps the bar fills in. If 0, bar is invisible. If 1, bar fades in fully.
///
/// Returns a tuple with the first component being the [`SDFValue`] for the progress bar,
/// and the second component being the fade intensity (`0.0..1.0`).
#[must_use]
fn draw_time_bar_discrete(
  uv: Vec2,
  aspect: Vec2,
  stroke: f32,
  progress: f32,
  index: u32,
  steps: u32,
) -> (SDFValue, f32) {
  if steps == 0 {
    return (SDFValue::FAR_OUTSIDE, 0.0);
  }

  let steps_f = steps as f32;
  // Determine how many steps should be visible, fractionally.
  // e.g., progress=0.75, steps=2 -> target_step_float = 1.5 (1 full, 1 half-faded)
  let target_step_float = progress * steps_f;
  let fade_start_step = target_step_float.floor();
  let fade_end_step = fade_start_step + 1.0;
  let mut fade_in_percentage = target_step_float.fract();

  let sdf_value = sdf_progress_bar(uv, aspect, stroke, fade_end_step / steps_f, index);

  let bar_full_potential_width = aspect.x * 2.0;
  let step_width = bar_full_potential_width / steps_f;
  let segment_left_bound = -aspect.x + step_width * fade_start_step;
  let segment_right_bound = -aspect.x + step_width * fade_end_step;
  if uv.x < segment_left_bound {
    fade_in_percentage = 1.0;
  }
  if uv.x > segment_right_bound {
    fade_in_percentage = 0.0;
  }

  (sdf_value, fade_in_percentage)
}

/// Alpha compositing using "over" operator.
/// https://en.wikipedia.org/wiki/Alpha_compositing
fn composite_layers<const N: usize>(overlay_colors: &[Vec4; N]) -> Vec4 {
  // Start with a fully opaque black background.
  let mut color_bg = Vec4::new(0.0, 0.0, 0.0, 1.0);
  for i in 0..overlay_colors.len() {
    let color_fg = overlay_colors[i];
    let color_fg_rgb = color_fg.xyz();
    let alpha_fg = color_fg.w;
    if alpha_fg <= 0.0 {
      continue; // Skip fully transparent layers.
    }

    let color_bg_rgb = color_bg.xyz();
    let alpha_bg = color_bg.w;

    let alpha_final = alpha_bg + alpha_fg - alpha_bg * alpha_fg;

    // Composite using the "over" operator.
    color_bg = ((color_fg_rgb * alpha_fg + color_bg_rgb * alpha_bg * (1.0 - alpha_fg))
      / alpha_final)
      .extend(alpha_final);
  }
  return color_bg;
}

const SHOW_TIME_BAR: bool = true;

impl Inputs {
  pub fn main_image(&self, frag_color: &mut Vec4, frag_coord: Vec2) {
    // Get screen dimensions as Vec2.
    let screen_xy = self.resolution.xy();
    // Determine the shorter dimension of the screen.
    let shorter_dim = screen_xy.min_element();

    let mut debug = false;
    // if mouse is pressed, enable debug mode
    if self.mouse.z > 0.0 {
      debug = true;
    }

    let debug_zoom = 10.0;
    let debug_translate = vec2(0.0, 9.0);

    let mut debug_zoom = 5.0;
    let mut debug_translate = vec2(0.0, 4.5);

    if !debug {
      debug_zoom = 1.0;
      debug_translate = vec2(0.0, 0.0);
    }

    // Compute normalized pixel coordinates.
    // This maps the center of the screen to (0,0) and the shortest side to [-1,1].
    // Aspect ratio is preserved.
    let uv = (frag_coord - screen_xy * 0.5) / shorter_dim * 2.0 * debug_zoom - debug_translate;
    let uv = (frag_coord - screen_xy * 0.5) / shorter_dim * 2.0 * debug_zoom - debug_translate;
    let aa_width = 2.0 / screen_xy.max_element();

    let aspect = screen_xy / shorter_dim;

    let mut black_alpha: f32 = 0.0;
    let mut debug_red_alpha: f32 = 0.0;
    let mut debug_blue_alpha: f32 = 0.0;
    let mut debug_green_alpha: f32 = 0.0;

    let outline_stroke = 0.05; // Width of the outline stroke.
    let m_viewport_rect = sdf_box_filled(uv, aspect).to_outline(outline_stroke);
    debug_red_alpha = debug_red_alpha.max(m_viewport_rect.to_alpha(aa_width));

    let target_radius = 0.2;
    let target_stroke = 0.05;
    let period = 8.0; //4.0; // seconds
    let period = 4.0; // seconds
    let t_master = (self.time / period).fract();
    //let t_master = 0.95;
    //let t_master = 0.8;
    //let t_master = 0.6;
    //let t_master = 0.5;
    let t_master_offset = offset_loop_time(t_master, 0.5);
    let t_outer_circle_radi = exp_time(remap_time(t_master, 0.3, 1.0), -0.4);

    let t_rotation = exp_time(t_master, 0.8);
    let t_trail_delayed = remap_time(t_master, 0.4, 1.0);
    let t_trail = exp_time(t_trail_delayed, 0.4);
    let t_assist_circle_delayed = remap_time(t_master, 0.5, 0.95);
    let t_assist_circle = exp_time(t_assist_circle_delayed, 0.4);
    //let t_exp = 1.0 - (-5.0 * t).exp();

    // rotating circles
    let num_circles = 12;
    let angle_between_circles = 2.0 * PI / num_circles as f32;
    let mut distance_for_main_circle = 0.0;
    for i in 0..num_circles {
      let circle_angle = angle_between_circles * i as f32;
      let distance_candidate = calculate_initial_distance_for_main_circle_center(
        aspect,
        target_radius + target_stroke / 2.0,
        circle_angle,
      );
      if let Some(distance_candidate) = distance_candidate {
        distance_for_main_circle = distance_candidate.max(distance_for_main_circle);
      }
    }
    // move from bottom_middle to center
    let middle_circle_start_radius = distance_for_main_circle;
    let middle_circle_radius =
      mix(middle_circle_start_radius, 0.0, t_master * t_master).max(target_radius);
    let middle_circle_start_position = Vec2::new(0.0, -distance_for_main_circle);

    let middle_circle_position = mix(middle_circle_start_position, Vec2::ZERO, t_master);
    let middle_circle_position = rotating_discrete_circle(
      middle_circle_start_position / 2.0, // maybe without /2.0
      middle_circle_start_radius / 2.0,
      -t_master * PI, // we only want half the rotation
      4,
      3,
    )
    .position;
    let middle_circle_radius = middle_circle_position.y.abs().max(target_radius);

    let m_middle_circle_position_path = sdf_circle_filled(
      uv - (middle_circle_start_position / 2.0),
      middle_circle_start_radius / 2.0,
    )
    .to_outline(target_stroke / 2.0);
    debug_blue_alpha = debug_blue_alpha.max(m_middle_circle_position_path.to_alpha(aa_width));

    let middle_circle_moved_distance =
      (middle_circle_start_position - middle_circle_position).length();
    let outer_circle_outer_radius = mix(target_radius, 0.0, t_outer_circle_radi);
    // (middle_circle_start_radius + target_radius)
    //  - (middle_circle_radius + middle_circle_moved_distance);
    let trail_angular_extent = mix(0.0, angle_between_circles, t_trail);
    let outer_circle_fade = mix(1.0, 0.4, t_trail);

    // y adjust to follow the middle circle
    //middle_circle_position.y -=
    //  (middle_circle_position.y + middle_circle_radius) * (1.0 - t_master);

    let m_start_circle = sdf_circle_filled(uv, target_radius).to_outline(target_stroke);
    debug_red_alpha = debug_red_alpha.max(m_start_circle.to_alpha(aa_width));
    let m_middle_circle_path = sdf_circle_filled(
      uv - middle_circle_start_position,
      middle_circle_start_radius,
    )
    .to_outline(target_stroke / 2.0);
    debug_red_alpha = debug_red_alpha.max(m_middle_circle_path.to_alpha(aa_width));

    let m_middle_circle_outline =
      sdf_circle_filled(uv - middle_circle_position, middle_circle_radius)
        .to_outline(target_stroke / 2.0);
    debug_blue_alpha = debug_blue_alpha.max(m_middle_circle_outline.to_alpha(aa_width));

    for i in 0..num_circles {
      // Compute the position of the circle based on the angle and radius.
      let outer_discrete_circle = rotating_discrete_circle(
        middle_circle_position,
        middle_circle_radius,
        -t_master * PI - t_rotation * TWO_PI * 5.0,
        num_circles,
        i,
      );

      let outer_circle_inner_radius = (outer_circle_outer_radius - target_stroke / 2.0).max(0.0);
      let outer_circle_outer_radius = outer_circle_outer_radius + target_stroke / 2.0;
      let m = sdf_arc_outline(
        uv - middle_circle_position,
        outer_discrete_circle.angle - trail_angular_extent / 2.0,
        outer_discrete_circle.angle + trail_angular_extent / 2.0,
        middle_circle_radius,
        outer_circle_inner_radius,
        outer_circle_outer_radius,
        outer_discrete_circle.angle,
        outer_circle_fade,
      );
      black_alpha = black_alpha.max(m.0.to_alpha(aa_width) * (m.1 + t_assist_circle).min(1.0));
      // * (m.y + 6.0 / 256.0));

      let m = sdf_circle_filled(uv - middle_circle_position, middle_circle_radius)
        .to_outline(outer_circle_outer_radius);
      black_alpha = black_alpha.max(m.to_alpha(aa_width) * t_assist_circle);
    }

    if debug && false {
      let sdf_arc_test = sdf_arc_outline(uv, -PI / 4.0, PI / 4.0, 1.0, 0.0, 0.5, -PI / 4.0, 1.0);
      debug_green_alpha =
        debug_green_alpha.max(sdf_arc_test.0.offset(0.05).to_alpha(aa_width) * sdf_arc_test.1);
      debug_red_alpha = debug_red_alpha.max(sdf_arc_test.0.to_alpha(aa_width) * sdf_arc_test.1);
    }

    if !debug {
      debug_blue_alpha = 0.0;
      debug_red_alpha = 0.0;
      debug_green_alpha = 0.0;
    }

    if SHOW_TIME_BAR {
      let m_master_time_bar = sdf_progress_bar(
        uv,
        aspect,
        target_stroke,
        t_master,
        0, // index
      );
      debug_green_alpha = debug_green_alpha.max(m_master_time_bar.to_alpha(aa_width));
      let m_master_time_bar = draw_time_bar_discrete(
        uv,
        aspect,
        target_stroke,
        t_master,
        1, // index
        10,
      );
      debug_green_alpha =
        debug_green_alpha.max(m_master_time_bar.0.to_alpha(aa_width) * m_master_time_bar.1);

      let m_master_time_offset_bar = sdf_progress_bar(
        uv,
        aspect,
        target_stroke,
        t_master_offset,
        2, // index
      );
      debug_red_alpha = debug_red_alpha.max(m_master_time_offset_bar.to_alpha(aa_width));
    }

    let color_background = Vec4::ONE;
    let color_black = Vec4::new(0.0, 0.0, 0.0, black_alpha);
    let color_red = Vec4::new(1.0, 0.0, 0.0, debug_red_alpha * 0.5);
    let color_blue = Vec4::new(0.0, 0.0, 1.0, debug_blue_alpha * 0.5);
    let color_green = Vec4::new(0.0, 1.0, 0.0, debug_green_alpha * 0.5);

    let color_rgb = composite_layers(&[
      color_background,
      color_black,
      color_red,
      color_blue,
      color_green,
    ]);

    // Output final pixel color with alpha = 1.0.
    *frag_color = color_rgb;
  }
}
