//! Ported to Rust from <https://www.shadertoy.com/view/ttKGDt>

use crate::shader_prelude::*;

pub const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
    name: "Phantom Star",
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

fn rot(a: f32) -> Mat2 {
    let c: f32 = a.cos();
    let s: f32 = a.sin();
    Mat2::from_cols_array(&[c, s, -s, c])
}

fn pmod(p: Vec2, r: f32) -> Vec2 {
    let mut a: f32 = p.x.atan2(p.y) + PI / r;
    let n: f32 = TAU / r;
    a = (a / n).floor() * n;
    rot(-a).transpose() * p
}

fn box_(p: Vec3, b: Vec3) -> f32 {
    let d: Vec3 = p.abs() - b;
    d.x.max(d.y.max(d.z)).min(0.0) + d.max(Vec3::ZERO).length()
}

impl Inputs {
    fn ifs_box(&self, mut p: Vec3) -> f32 {
        for _ in 0..5 {
            p = p.abs() - Vec3::splat(1.0);
            p = (rot(self.time * 0.3).transpose() * p.xy()).extend(p.z);
            p = (rot(self.time * 0.1).transpose() * p.xz())
                .extend(p.y)
                .xzy();
        }
        p = (rot(self.time).transpose() * p.xz()).extend(p.y).xzy();
        box_(p, vec3(0.4, 0.8, 0.3))
    }

    fn map(&self, p: Vec3, _c_pos: Vec3) -> f32 {
        let mut p1: Vec3 = p;
        p1.x = (p1.x - 5.0).rem_euclid(10.0) - 5.0;
        p1.y = (p1.y - 5.0).rem_euclid(10.0) - 5.0;
        p1.z = p1.z.rem_euclid(16.0) - 8.0;
        p1 = pmod(p1.xy(), 5.0).extend(p1.z);
        self.ifs_box(p1)
    }

    fn main_image(&mut self, frag_color: &mut Vec4, frag_coord: Vec2) {
        let p: Vec2 =
            (frag_coord * 2.0 - self.resolution.xy()) / self.resolution.x.min(self.resolution.y);

        let c_pos: Vec3 = vec3(0.0, 0.0, -3.0 * self.time);
        // let c_pos: Vec3 = vec3(0.3 * (self.time * 0.8).sin(), 0.4 * (self.time * 0.3).cos(), -6.0 * self.time,);
        let c_dir: Vec3 = vec3(0.0, 0.0, -1.0).normalize();
        let c_up: Vec3 = vec3(self.time.sin(), 1.0, 0.0);
        let c_side: Vec3 = c_dir.cross(c_up);

        let ray: Vec3 = (c_side * p.x + c_up * p.y + c_dir).normalize();

        // Phantom Mode https://www.shadertoy.com/view/MtScWW by aiekick
        let mut acc: f32 = 0.0;
        let mut acc2: f32 = 0.0;
        let mut t: f32 = 0.0;

        for _ in 0..99 {
            let pos: Vec3 = c_pos + ray * t;
            let mut dist: f32 = self.map(pos, c_pos);
            dist = dist.abs().max(0.02);
            let mut a: f32 = (-dist * 3.0).exp();
            if (pos.length() + 24.0 * self.time).rem_euclid(30.0) < 3.0 {
                a *= 2.0;
                acc2 += a;
            }
            acc += a;
            t += dist * 0.5;
        }

        let col: Vec3 = vec3(
            acc * 0.01,
            acc * 0.011 + acc2 * 0.002,
            acc * 0.012 + acc2 * 0.005,
        );
        *frag_color = col.extend(1.0 - t * 0.03);
    }
}
