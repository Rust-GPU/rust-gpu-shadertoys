//! Ported to Rust from <https://www.shadertoy.com/view/XsfGRn>
//!
//! Original comment:
//! ```glsl
//! // Created by inigo quilez - iq/2013
//! // License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
//! ```

use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition { name: "Hearts" };

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
    fn main_image(&self, frag_color: &mut Vec4, frag_coord: Vec2) {
        let mut p: Vec2 =
            (2.0 * frag_coord - self.resolution.xy()) / (self.resolution.xy().min_element());

        // background color
        let bcol: Vec3 = vec3(1.0, 0.8, 0.7 - 0.07 * p.y) * (1.0 - 0.25 * p.length());

        // animate
        let tt: f32 = self.time.rem_euclid(1.5) / 1.5;
        let mut ss: f32 = tt.powf(0.2) * 0.5 + 0.5;
        ss = 1.0 + ss * 0.5 * (tt * TAU * 3.0 + p.y * 0.5).sin() * (-tt * 4.0).exp();
        p *= vec2(0.5, 1.5) + ss * vec2(0.5, -0.5);

        // shape
        let r: f32;
        let d: f32;

        if false {
            p *= 0.8;
            p.y = -0.1 - p.y * 1.2 + p.x.abs() * (1.0 - p.x.abs());
            r = p.length();
            d = 0.5;
        } else {
            p.y -= 0.25;
            let a: f32 = p.x.atan2(p.y) / PI;
            r = p.length();
            let h: f32 = a.abs();
            d = (13.0 * h - 22.0 * h * h + 10.0 * h * h * h) / (6.0 - 5.0 * h);
        }

        // color
        let mut s: f32 = 0.75 + 0.75 * p.x;
        s *= 1.0 - 0.4 * r;
        s = 0.3 + 0.7 * s;
        s *= 0.5 + 0.5 * (1.0 - (r / d).clamp(0.0, 1.0)).powf(0.1);
        let hcol: Vec3 = vec3(1.0, 0.5 * r, 0.3) * s;

        let col: Vec3 = mix(bcol, hcol, smoothstep(-0.01, 0.01, d - r));

        *frag_color = col.extend(1.0);
    }
}
