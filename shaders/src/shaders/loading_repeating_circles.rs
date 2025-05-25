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
/// Anti-aliasing width for edges.
const AA_WIDTH: f32 = 0.01;

/// An SDF value that can be negative (inside the shape) or positive (outside the shape).
#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct SDFValue(f32);
impl SDFValue {
  /// Creates a new SDFValue.
  pub fn new(value: f32) -> Self {
    SDFValue(value)
  }

  /// Returns the raw f32 distance.
  pub fn value(self) -> f32 {
    self.0
  }

  /// Returns `true` if the SDF value is inside the shape (negative).
  pub fn is_inside(self) -> bool {
    self.0 < 0.0
  }

  /// Returns `true` if the SDF value is outside the shape (positive).
  pub fn is_outside(self) -> bool {
    self.0 > 0.0
  }

  /// Converts the SDF value to an alpha value for anti-aliasing.
  /// Alpha is 1.0 deep inside, 0.0 deep outside, and smooth in between.
  /// The transition happens from `AA_WIDTH` (alpha 0) to `-AA_WIDTH` (alpha 1).
  pub fn to_alpha(self) -> f32 {
    return smoothstep(AA_WIDTH, -AA_WIDTH, self.0);
  }

  /// Insets the shape by a given thickness.
  /// This is done by shrinking the shape and then calculating the difference.
  pub fn inset(self, amount: f32) -> Self {
    Self(self.0.max(-(self.0 + amount)))
  }

  /// Expands (if amount > 0) or shrinks (if amount < 0) the shape.
  /// This is equivalent to subtracting from the distance value.
  pub fn offset(&self, amount: f32) -> Self {
    Self(self.0 - amount)
  }

  /// Takes the absolute value of the SDF, effectively creating an infinitely thin shell
  /// on the surface of the original shape.
  pub fn shell(self) -> Self {
    Self(self.0.abs())
  }

  /// Creates an outline (hollow shape) from the SDF.
  pub fn to_outline(self, thickness: f32) -> Self {
    Self(self.0.abs() - thickness)
  }

  /// Difference operation (self - other). Result is inside if inside self AND outside other.
  /// Equivalent to Intersection(self, Invert(other)).
  pub fn difference(self, other: Self) -> Self {
    Self(self.0.max(-other.0))
  }

  /// Union operation (self U other). Result is inside if inside self OR inside other.
  pub fn union(self, other: Self) -> Self {
    Self(self.0.min(other.0))
  }

  /// Intersection operation (self ∩ other). Result is inside if inside self AND inside other.
  pub fn intersection(self, other: Self) -> Self {
    Self(self.0.max(other.0))
  }

  /// Inverts the SDF (inside becomes outside and vice-versa).
  pub fn invert(self) -> Self {
    Self(-self.0)
  }
}

/// Calculates the distance from the origin to the center of the initial main circle so that
/// at time `0`, only one border circle is visible, with the others just touching the sides/bottom of the viewport.
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
/// * `center_shape`: Center of the arc for the arc spine.
/// * `start_angle`, `end_angle`: Defines the arc of the spine (in radians).
///                               The arc is drawn CCW from `start_angle`.
/// * `spine_radius`: Radius of the arc spine.
/// * `stroke`: Width of the arc body.
///
/// Returns the [`SDFValue`] for the arc.
fn sdf_arc_filled(
  uv: Vec2,
  center_shape: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  stroke: f32,
) -> SDFValue {
  let half_stroke = stroke * 0.5;
  let p = uv - center_shape;

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
    sdf_value = (p - arc_spine_point_local).length() - half_stroke;
  } else if (effective_arc_length_angle - TWO_PI).abs() < EPSILON {
    // Case 2: Arc is a full circle (annulus)
    sdf_value = (p.length() - spine_radius).abs() - half_stroke;
  } else {
    // Case 3: Arc is a partial arc (with rounded caps)
    let mid_angle_of_arc = start_angle + effective_arc_length_angle / 2.0;
    let rot_angle_for_symmetry = -mid_angle_of_arc;

    let cs_rot = rot_angle_for_symmetry.cos();
    let sn_rot = rot_angle_for_symmetry.sin();

    let p_sym_x = p.x * cs_rot - p.y * sn_rot;
    let p_sym_y = p.x * sn_rot + p.y * cs_rot;
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
/// * `center`: Center of the circular arc.
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
fn arc_fade_out(
  uv: Vec2,
  center: Vec2,
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

  let dir = (uv - center).normalize();
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
/// * `center_shape`: Center of the arc for the arc spine.
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
/// Returns a `Vec2` with the first component being the [`SDFValue`] for the arc outline,
/// and the second component being the fade intensity.
fn sdf_arc_outline(
  uv: Vec2,
  center_shape: Vec2,
  start_angle: f32,
  end_angle: f32,
  spine_radius: f32,
  inner_radius: f32,
  outer_radius: f32,
  fade_center_angle: f32,
  opaque_percentage: f32,
) -> (SDFValue, f32) {
  let sdf_outer = sdf_arc_filled(
    uv,
    center_shape,
    start_angle,
    end_angle,
    spine_radius,
    outer_radius * 2.0,
  );
  let sdf_value = sdf_outer.inset(outer_radius - inner_radius);
  let fade_intensity = arc_fade_out(
    uv,
    center_shape,
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
/// * `center`: The center of the circle.
/// * `radius`: The radius of the circle.
///
/// Returns the [`SDFValue`] for the circle.
fn sdf_circle_filled(uv: Vec2, center: Vec2, radius: f32) -> SDFValue {
  let p = uv - center;
  let d = p.length() - radius;
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
fn exp_time(t: f32, c: f32) -> f32 {
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
fn exp_time_derivative(t: f32, c: f32) -> f32 {
  let c = c * 10.0;

  if c.abs() < EPSILON {
    return 1.0;
  }

  let numerator = c * (c * t).exp();
  let denominator = c.exp() - 1.0;
  numerator / denominator
}

fn offset_loop_time(t: f32, offset: f32) -> f32 {
  // Apply offset
  let offset_t = t + offset;
  // Wrap around to [0, 1]
  return offset_t.rem_euclid(1.0);
}

struct RotatingCircleResult {
  position: Vec2,
  angle: f32,
}

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

/// General purpose function to remap a time segment to `0..1`.
///
/// * `parent_t`: The main animation time, expected to be `0..1`.
/// * `start_time`: The point in parent_t (`0..1`) where this sub-animation should begin.
/// * `end_time`: The point in parent_t (`0..1`) where this sub-animation should end.
///
/// Returns: `0` before `start_time`, `1` after `end_time`, and a `0..1` ramp between them.
fn remap_time(parent_t: f32, start_time: f32, end_time: f32) -> f32 {
  if start_time >= end_time {
    // If start and end are the same, or invalid order we do an instant step.
    return if parent_t >= start_time { 1.0 } else { 0.0 };
  }
  let duration = end_time - start_time;
  return ((parent_t - start_time) / duration).clamp(0.0, 1.0);
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum Positioning {
  Centered,
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
}

/// SDF for a filled box.
///
/// * `uv`: The coordinates relative to the center of the box.
/// * `center`: The center of the box.
/// * `positioning`: How the box is positioned relative to `uv`.
/// * `half_dimensions`: half-width and half-height of the box.
///
/// Returns the signed distance from the box.
fn sdf_box_filled(uv: Vec2, center: Vec2, positioning: Positioning, half_dimensions: Vec2) -> f32 {
  let actual_box_center = match positioning {
    Positioning::Centered => center,
    Positioning::TopLeft => Vec2::new(center.x + half_dimensions.x, center.y - half_dimensions.y),
    Positioning::TopRight => Vec2::new(center.x - half_dimensions.x, center.y - half_dimensions.y),
    Positioning::BottomLeft => {
      Vec2::new(center.x + half_dimensions.x, center.y + half_dimensions.y)
    },
    Positioning::BottomRight => {
      Vec2::new(center.x - half_dimensions.x, center.y + half_dimensions.y)
    },
  };

  let p = uv - actual_box_center;
  let d = p.abs() - half_dimensions;
  let outside_distance = d.max(Vec2::ZERO).length();
  let inside_distance = d.x.max(d.y).min(0.0);

  outside_distance + inside_distance
}

/// Draws the outline of a rectangle using the signed distance function.
///
/// * `uv`: The coordinates relative to the center of the rectangle.
/// * `center`: The center of the box.
/// * `positioning`: How the box is positioned relative to `uv`.
/// * `half_dimensions`: half-width and half-height of the box.
/// * `stroke`: This parameter will define the width of the outline.
///
/// Returns the signed distance from the rectangle outline.
fn sdf_box_outline(
  uv: Vec2,
  center: Vec2,
  positioning: Positioning,
  half_dimensions: Vec2,
  stroke: f32,
) -> f32 {
  let distance_to_filled_box = sdf_box_filled(uv, center, positioning, half_dimensions);

  let half_stroke = stroke * 0.5;

  let distance_to_outline = distance_to_filled_box.abs() - half_stroke;

  distance_to_outline
}

/// Draws a filled progress bar rectangle based on time `t`.
/// This is a simplified version focusing on a basic filled rectangle.
/// Bars are stacked downwards from the top of the screen.
/// `uv`: coordinates relative to the center of the screen.
/// `aspect`: half-dimensions of the screen (half-width, half-height), e.g., (aspect_ratio, 1.0) or (1.0, 1.0/aspect_ratio).
/// `stroke`: This parameter will define the height of the time bar.
/// `aa_width`: width for anti-aliasing smooth transitions at the edges of the bar.
/// `t`: progress of the bar, from 0.0 (empty) to 1.0 (full).
/// `index`: vertical stacking index of the bar. Index 0 is the top-most bar.
/// Returns an alpha value (0.0 to 1.0) for the pixel, representing the bar's visibility.
fn draw_time_bar(uv: Vec2, aspect: Vec2, stroke: f32, aa_width: f32, t: f32, index: u32) -> f32 {
  // --- Bar Configuration ---
  // The `stroke` argument is interpreted as the desired height of the bar.
  let bar_height = stroke;

  // If the bar height is non-positive, it's invisible.
  if bar_height <= 0.0 {
    return 0.0;
  }

  // A small constant gap between stacked bars.
  const INTER_BAR_CONSTANT_GAP: f32 = 0.01; // Normalized screen units.

  // --- Bar Dimensions & Position ---
  // The bar spans the full width of the viewport.
  let bar_full_potential_width = aspect.x * 2.0;

  // Calculate the vertical position of the current bar.
  // `index = 0` is the top-most bar.
  // `aspect.y` is the Y-coordinate of the top edge of the screen.
  // Each subsequent bar (`index > 0`) is placed below the previous one.
  let bar_top_edge_y = aspect.y - (index as f32) * (bar_height + INTER_BAR_CONSTANT_GAP);
  let bar_vertical_center_y = bar_top_edge_y - bar_height / 2.0;

  // --- Fill Calculation ---
  // Clamp progress `t` to the [0, 1] range.
  let t_clamped = t.clamp(0.0, 1.0);

  // Calculate the current width of the filled portion of the bar.
  let current_filled_width = bar_full_potential_width * t_clamped;

  // If the bar is too thin to be rendered meaningfully (thinner than anti-aliasing width),
  // treat it as invisible to prevent rendering artifacts.
  if current_filled_width < aa_width || bar_height < aa_width {
    return 0.0;
  }

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
  let distance_to_fill_boundary = sdf_box_filled(
    uv_relative_to_fill_rect_center,
    Vec2::ZERO,
    Positioning::Centered,
    fill_rect_half_dims,
  );

  // Use `smoothstep` to create a smooth transition (anti-aliasing) at the edges.
  // `smoothstep(edge0, edge1, x)`:
  // Here, `edge0 = aa_width`, `edge1 = -aa_width`.
  // If `distance_to_fill_boundary` is less than `-aa_width` (deep inside), alpha is 1.0.
  // If `distance_to_fill_boundary` is greater than `aa_width` (far outside), alpha is 0.0.
  // Transition occurs between `-aa_width` and `aa_width`.
  let alpha = smoothstep(aa_width, -aa_width, distance_to_fill_boundary);

  alpha
}

/// Draws a filled progress bar rectangle that fills in discrete steps,
/// with new steps fading in.
/// Bars are stacked downwards from the top of the screen.
/// `uv`: coordinates relative to the center of the screen.
/// `aspect`: half-dimensions of the screen (half-width, half-height).
/// `stroke`: This parameter will define the height of the time bar.
/// `aa_width`: width for anti-aliasing smooth transitions at the edges of the bar.
/// `t`: overall progress of the bar, from 0.0 (empty) to 1.0 (full).
/// `index`: vertical stacking index of the bar. Index 0 is the top-most bar.
/// `steps`: the number of discrete steps the bar fills in. If 0, bar is invisible. If 1, bar fades in fully.
/// Returns an alpha value (0.0 to 1.0) for the pixel, representing the bar's visibility.
fn draw_time_bar_discrete(
  uv: Vec2,
  aspect: Vec2,
  stroke: f32,
  aa_width: f32,
  t: f32, // Overall progress
  index: u32,
  steps: u32,
) -> f32 {
  // --- Basic Validations ---
  // If bar has no height or no steps, it's invisible.
  if stroke <= 0.0 || steps == 0 {
    return 0.0;
  }

  // --- Bar Configuration ---
  let bar_height = stroke;
  // Consistent gap with draw_time_bar if used together.
  const INTER_BAR_CONSTANT_GAP: f32 = 0.01; // Normalized screen units.

  // --- Bar Dimensions & Position ---
  // Bar spans the full potential width of the viewport.
  let bar_full_potential_width = aspect.x * 2.0;
  // Calculate vertical position of the bar.
  let bar_top_edge_y = aspect.y - (index as f32) * (bar_height + INTER_BAR_CONSTANT_GAP);
  let bar_vertical_center_y = bar_top_edge_y - bar_height / 2.0;

  // --- Progress Calculation ---
  // Clamp overall progress `t` to the [0, 1] range.
  let t_clamped = t.clamp(0.0, 1.0);

  // If progress is effectively zero, bar is invisible.
  if t_clamped <= EPSILON {
    return 0.0;
  }

  // Calculate the render width of a single discrete step.
  let single_step_render_width = bar_full_potential_width / steps as f32;

  // Convert overall progress `t_clamped` into step-based progress.
  // e.g., t_clamped=0.6, steps=4 -> progress_in_num_steps=2.4
  // This means 2 steps are fully complete, and the 3rd step is 40% through its fade-in.
  let progress_in_num_steps = t_clamped * steps as f32;

  // Number of fully completed (solid) segments.
  // For progress_in_num_steps=2.4, num_solid_segments=2.
  let num_solid_segments = progress_in_num_steps.floor() as u32;

  // Fractional progress within the current fading-in segment (0.0 to ~1.0).
  // For progress_in_num_steps=2.4, current_segment_fade_progress=0.4.
  // This value determines the alpha of the fading segment.
  let current_segment_fade_progress = progress_in_num_steps.fract();

  let mut final_alpha: f32 = 0.0;

  // --- 1. Draw Fully Completed (Solid) Segments ---
  if num_solid_segments > 0 {
    let solid_part_width = num_solid_segments as f32 * single_step_render_width;

    // Ensure the solid part has a positive width to render.
    if solid_part_width > EPSILON {
      let solid_rect_half_dims = vec2(solid_part_width * 0.5, bar_height * 0.5);
      // Solid part starts at left edge (-aspect.x) and extends by solid_part_width.
      let solid_rect_center_x = -aspect.x + solid_part_width * 0.5;
      let solid_rect_center_pos = vec2(solid_rect_center_x, bar_vertical_center_y);

      // Calculate SDF for the solid part.
      let dist_to_solid_boundary = sdf_box_filled(
        uv,
        solid_rect_center_pos,
        Positioning::Centered,
        solid_rect_half_dims,
      );
      // Apply anti-aliasing.
      let alpha_solid_part = smoothstep(aa_width, -aa_width, dist_to_solid_boundary);
      final_alpha = final_alpha.max(alpha_solid_part);
    }
  }

  // --- 2. Draw the Currently Fading-In Segment ---
  // This segment is drawn if:
  //   a) Not all steps are already solid (num_solid_segments < steps).
  //   b) There's some progress into this new segment (current_segment_fade_progress > EPSILON).
  if num_solid_segments < steps && current_segment_fade_progress > EPSILON {
    // The fading segment starts where the solid segments (if any) ended.
    let fading_segment_start_x = -aspect.x + (num_solid_segments as f32 * single_step_render_width);

    let fading_rect_half_dims = vec2(single_step_render_width * 0.5, bar_height * 0.5);
    // Center of this single fading segment.
    let fading_rect_center_x = fading_segment_start_x + single_step_render_width * 0.5;
    let fading_rect_center_pos = vec2(fading_rect_center_x, bar_vertical_center_y);

    // Calculate SDF for the shape of the fading segment.
    let dist_to_fading_boundary = sdf_box_filled(
      uv,
      fading_rect_center_pos,
      Positioning::Centered,
      fading_rect_half_dims,
    );
    // Base alpha for the shape with anti-aliasing.
    let base_alpha_fading_shape = smoothstep(aa_width, -aa_width, dist_to_fading_boundary);

    // Modulate shape alpha by the fade-in progress.
    let alpha_fading_part = base_alpha_fading_shape * current_segment_fade_progress;
    final_alpha = final_alpha.max(alpha_fading_part);
  }

  // Edge case: t_clamped = 1.0 (fully complete)
  // - progress_in_num_steps = steps as f32
  // - num_solid_segments = steps
  // - current_segment_fade_progress = 0.0 (or very close to 0 due to precision)
  // Solid part: Draws all 'steps' segments fully.
  // Fading part: Condition 'num_solid_segments < steps' is false, so it's skipped.
  // Result: The entire bar is solid, which is correct.

  final_alpha
}

/// Alpha compositing using "over" operator.
/// https://en.wikipedia.org/wiki/Alpha_compositing
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

const SHOW_TIME_BAR: bool = true;

impl Inputs {
  pub fn main_image(&self, frag_color: &mut Vec4, frag_coord: Vec2) {
    // Get screen dimensions as Vec2.
    let screen_xy = self.resolution.xy();
    // Determine the shorter dimension of the screen.
    let shorter_dim = screen_xy.min_element();

    let mut DEBUG = false;
    // if mouse is pressed, enable debug mode
    if self.mouse.z > 0.0 {
      DEBUG = true;
    }

    let debug_zoom = 10.0;
    let debug_translate = vec2(0.0, 9.0);

    let mut debug_zoom = 5.0;
    let mut debug_translate = vec2(0.0, 4.5);

    if !DEBUG {
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
    let m_viewport_rect = sdf_box_outline(
      uv,
      Vec2::ZERO,
      Positioning::Centered,
      aspect,
      outline_stroke,
    );
    debug_red_alpha = debug_red_alpha.max(1.0 - smoothstep(0.0, aa_width, m_viewport_rect));

    let center = Vec2::ZERO;
    let bottom_middle = vec2(0.0, -aspect.y);
    let top_middle = vec2(0.0, aspect.y);
    let left_middle = vec2(-aspect.x, 0.0);
    let right_middle = vec2(aspect.x, 0.0);

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
    let mut H = 0.0;
    for i in 0..num_circles {
      let circle_angle = angle_between_circles * i as f32;
      let H_candidate = calculate_initial_distance_for_main_circle_center(
        aspect,
        target_radius + target_stroke / 2.0,
        circle_angle,
      );
      if let Some(H_candidate) = H_candidate {
        H = H_candidate.max(H);
      }
    }
    // move from bottom_middle to center
    let middle_circle_start_radius = H;
    let middle_circle_radius =
      mix(middle_circle_start_radius, 0.0, t_master * t_master).max(target_radius);
    let middle_circle_start_position = Vec2::new(0.0, -H);

    let mut middle_circle_position = mix(middle_circle_start_position, center, t_master);
    let mut middle_circle_position = rotating_discrete_circle(
      middle_circle_start_position / 2.0, // maybe without /2.0
      middle_circle_start_radius / 2.0,
      -t_master * PI, // we only want half the rotation
      4,
      3,
    )
    .position;
    let middle_circle_radius = middle_circle_position.y.abs().max(target_radius);

    let m_middle_circle_position_path = sdf_circle_filled(
      uv,
      middle_circle_start_position / 2.0,
      middle_circle_start_radius / 2.0,
    )
    .to_outline(target_stroke / 2.0);
    debug_blue_alpha = debug_blue_alpha.max(m_middle_circle_position_path.to_alpha());

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

    let m_start_circle = sdf_circle_filled(uv, Vec2::ZERO, target_radius).to_outline(target_stroke);
    debug_red_alpha = debug_red_alpha.max(m_start_circle.to_alpha());
    let m_middle_circle_path =
      sdf_circle_filled(uv, middle_circle_start_position, middle_circle_start_radius)
        .to_outline(target_stroke / 2.0);
    debug_red_alpha = debug_red_alpha.max(m_middle_circle_path.to_alpha());

    let m_middle_circle_outline =
      sdf_circle_filled(uv, middle_circle_position, middle_circle_radius)
        .to_outline(target_stroke / 2.0);
    debug_blue_alpha = debug_blue_alpha.max(m_middle_circle_outline.to_alpha());

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
      black_alpha = black_alpha.max(m.0.to_alpha() * (m.1 + t_assist_circle).min(1.0));
      // * (m.y + 6.0 / 256.0));

      let m = sdf_circle_filled(uv, middle_circle_position, middle_circle_radius)
        .to_outline(outer_circle_outer_radius * 2.0);
      black_alpha = black_alpha.max(m.to_alpha() * t_assist_circle);
    }

    if DEBUG {
      let sdf_arc_test = sdf_arc_outline(
        uv,
        center,
        -PI / 4.0,
        PI / 4.0,
        1.0,
        0.1,
        0.5,
        -PI / 4.0,
        1.0,
      );
      debug_green_alpha =
        debug_green_alpha.max(sdf_arc_test.0.offset(0.05).to_alpha() * sdf_arc_test.1);
      debug_red_alpha = debug_red_alpha.max(sdf_arc_test.0.to_alpha() * sdf_arc_test.1);
    }

    if !DEBUG {
      debug_blue_alpha = 0.0;
      debug_red_alpha = 0.0;
      debug_green_alpha = 0.0;
    }

    if SHOW_TIME_BAR {
      let m_master_time_bar = draw_time_bar(
        uv,
        aspect,
        target_stroke,
        aa_width,
        t_master,
        0, // index
      );
      debug_green_alpha = debug_green_alpha.max(m_master_time_bar);
      let m_master_time_bar = draw_time_bar_discrete(
        uv,
        aspect,
        target_stroke,
        aa_width,
        t_master,
        1, // index
        10,
      );
      debug_green_alpha = debug_green_alpha.max(m_master_time_bar);

      let m_master_time_offset_bar = draw_time_bar(
        uv,
        aspect,
        target_stroke,
        aa_width,
        t_master_offset,
        2, // index
      );
      debug_red_alpha = debug_red_alpha.max(m_master_time_offset_bar);
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
