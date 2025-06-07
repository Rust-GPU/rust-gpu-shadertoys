//! Ported to Rust from <https://www.shadertoy.com/view/MsfGzM>
//!
//! Original comment:
//! ```glsl
//! // Created by inigo quilez - iq/2013
//! // License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
//! ```

use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition { name: "Two Tweets" };

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

impl Inputs {
    fn f(&self, mut p: Vec3) -> f32 {
        p.z += self.time;
        (Vec3::splat(0.05 * (9. * p.y * p.x).cos()) + vec3(p.x.cos(), p.y.cos(), p.z.cos())
            - Vec3::splat(0.1 * (9. * (p.z + 0.3 * p.x - p.y)).cos()))
        .length()
            - 1.
    }

    fn main_image(&self, c: &mut Vec4, p: Vec2) {
        let d: Vec3 = Vec3::splat(0.5) - p.extend(1.0) / self.resolution.x;
        let mut o: Vec3 = d;
        for _ in 0..128 {
            o += self.f(o) * d;
        }
        *c = ((self.f(o - d) * vec3(0.0, 1.0, 2.0)
            + Vec3::splat(self.f(o - Vec3::splat(0.6))) * vec3(2.0, 1.0, 0.0))
        .abs()
            * (1. - 0.1 * o.z))
            .extend(1.0);
    }
}
