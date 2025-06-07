//! Ported to Rust from <https://www.shadertoy.com/view/llXSzX>

use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
  name: "Moving Square",
};

pub fn shader_fn(render_instruction: &ShaderInput, render_result: &mut ShaderResult) {
  let color = &mut render_result.color;
  let &ShaderInput {
    resolution,
    time,
    frag_coord,
    ..
  } = render_instruction;
  Inputs { resolution, time }.main_image(color, frag_coord);
}

struct Inputs {
  resolution: Vec3,
  time: f32,
}

fn rect(uv: Vec2, pos: Vec2, r: f32) -> Vec4 {
  let re_c: Vec2 = (uv - pos).abs();
  let dif1: Vec2 = re_c - Vec2::splat(r / 2.);
  let dif2: Vec2 = (re_c - Vec2::splat(r / 2.)).clamp(Vec2::ZERO, Vec2::ONE);
  let d1: f32 = (dif1.x + dif1.y).clamp(0.0, 1.0);
  let _d2: f32 = (dif2.x + dif2.y).clamp(0.0, 1.0);

  Vec4::splat(d1)
}

impl Inputs {
  fn main_image(&self, frag_color: &mut Vec4, frag_coord: Vec2) {
    let mut uv: Vec2 = frag_coord;
    let t: f32 = self.time.sin();

    let c: Vec2 = self.resolution.xy() * 0.5; // + sin(iTime) * 50.;

    uv = Mat2::from_cols_array(&[t.cos(), -t.sin(), t.sin(), t.cos()]) * (uv - c) + c;

    *frag_color = rect(uv, c, (self.time * 10.).sin() * 50. + 50.);
    *frag_color *= vec3(0.5, 0.2, 1.).extend(1.);
    *frag_color += rect(uv, c, self.time.sin() * 50. + 50.);
    *frag_color *= vec3(0.5, 0.8, 1.).extend(1.);
  }
}
