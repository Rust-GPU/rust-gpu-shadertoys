//! Ported to Rust from <https://www.shadertoy.com/view/MtX3Ws>
//!
//! Original comment:
//! ```glsl
//! // License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
//! // Created by S. Guillitte 2015
//! ```

use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
    name: "Playing Marble",
};

pub fn shader_fn(render_instruction: &ShaderInput, render_result: &mut ShaderResult) {
    let color = &mut render_result.color;
    let &ShaderInput {
        resolution,
        time,
        frag_coord,
        mouse,
        ..
    } = render_instruction;
    Inputs {
        resolution,
        time,
        mouse,
        channel0: RgbCube {
            alpha: 1.0,
            intensity: 1.0,
        },
    }
    .main_image(color, frag_coord);
}

struct Inputs<C0> {
    resolution: Vec3,
    time: f32,
    mouse: Vec4,
    channel0: C0,
}

const ZOOM: f32 = 1.0;

fn _cmul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x)
}
fn csqr(a: Vec2) -> Vec2 {
    vec2(a.x * a.x - a.y * a.y, 2. * a.x * a.y)
}

fn rot(a: f32) -> Mat2 {
    Mat2::from_cols_array(&[a.cos(), a.sin(), -a.sin(), a.cos()])
}

//from iq
fn i_sphere(ro: Vec3, rd: Vec3, sph: Vec4) -> Vec2 {
    let oc: Vec3 = ro - sph.xyz();
    let b: f32 = oc.dot(rd);
    let c: f32 = oc.dot(oc) - sph.w * sph.w;
    let mut h: f32 = b * b - c;
    if h < 0.0 {
        return Vec2::splat(-1.0);
    }
    h = h.sqrt();
    vec2(-b - h, -b + h)
}

fn map(mut p: Vec3) -> f32 {
    let mut res: f32 = 0.0;
    let c: Vec3 = p;
    for _ in 0..10 {
        p = 0.7 * p.abs() / p.dot(p) - Vec3::splat(0.7);
        p = csqr(p.yz()).extend(p.x).zxy();
        p = p.zxy();
        res += (-19.0 * p.dot(c).abs()).exp();
    }
    res / 2.0
}

impl<C0: SampleCube> Inputs<C0> {
    fn raymarch(&self, ro: Vec3, rd: Vec3, tminmax: Vec2) -> Vec3 {
        let mut t: f32 = tminmax.x;
        let dt: f32 = 0.02;
        //let dt: f32 = 0.2 - 0.195 * (self.time * 0.05).cos(); //animated
        let mut col: Vec3 = Vec3::ZERO;
        let mut c: f32 = 0.0;
        for _ in 0..64 {
            t += dt * (-2.0 * c).exp();
            if t > tminmax.y {
                break;
            }
            let _pos: Vec3 = ro + t * rd;

            c = map(ro + t * rd);

            col = 0.99 * col + 0.08 * vec3(c * c, c, c * c * c); //green

            // col = 0.99 * col + 0.08 * vec3(c * c * c, c * c, c); //blue
        }
        col
    }
    fn main_image(&mut self, frag_color: &mut Vec4, frag_coord: Vec2) {
        let time: f32 = self.time;
        let q: Vec2 = frag_coord / self.resolution.xy();
        let mut p: Vec2 = Vec2::splat(-1.0) + 2.0 * q;
        p.x *= self.resolution.x / self.resolution.y;
        let mut m: Vec2 = Vec2::ZERO;
        if self.mouse.z > 0.0 {
            m = self.mouse.xy() / self.resolution.xy() * 3.14;
        }
        m = m - Vec2::splat(0.5);

        // camera

        let mut ro: Vec3 = ZOOM * Vec3::splat(4.0);
        ro = (rot(m.y).transpose() * ro.yz()).extend(ro.x).zxy();
        ro = (rot(m.x + 0.1 * time).transpose() * ro.xz())
            .extend(ro.y)
            .xzy();
        let ta: Vec3 = Vec3::ZERO;
        let ww: Vec3 = (ta - ro).normalize();
        let uu: Vec3 = (ww.cross(vec3(0.0, 1.0, 0.0))).normalize();
        let vv: Vec3 = (uu.cross(ww)).normalize();
        let rd: Vec3 = (p.x * uu + p.y * vv + 4.0 * ww).normalize();

        let tmm: Vec2 = i_sphere(ro, rd, vec4(0.0, 0.0, 0.0, 2.0));
        // raymarch
        let mut col: Vec3 = self.raymarch(ro, rd, tmm);
        if tmm.x < 0.0 {
            col = self.channel0.sample_cube(rd).xyz();
        } else {
            let mut nor: Vec3 = (ro + tmm.x * rd) / 2.;
            nor = rd.reflect(nor);
            let fre: f32 = (0.5 + nor.dot(rd).clamp(0.0, 1.0)).powf(3.0) * 1.3;
            col += self.channel0.sample_cube(nor).xyz() * fre;
        }

        //shade

        col = 0.5 * (Vec3::ONE + col).ln();
        col = col.clamp(Vec3::ZERO, Vec3::ONE);

        *frag_color = col.extend(1.0);
    }
}
