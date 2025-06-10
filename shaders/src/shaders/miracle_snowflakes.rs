//! Ported to Rust from <https://www.shadertoy.com/view/Xsd3zf>
//!
//! Original comment:
//! ```glsl
//! /*
//! //
//! /* Panteleymonov Aleksandr Konstantinovich 2015
//! //
//! // if i write this string my code will be 0 chars, :) */
//! */
//! ```

use crate::shader_prelude::*;

const SHADER_DEFINITION: ShaderDefinition = ShaderDefinition {
    name: "Playing Marble",
};

impl Shader for ShaderMiracleSnowflakes<'_> {
    const SHADER_DEFINITION: &'static ShaderDefinition = &SHADER_DEFINITION;

    fn shader_fn(shader_input: &ShaderInput, shader_output: &mut ShaderResult) {
        let frag_color = &mut shader_output.color;
        let &ShaderInput { frag_coord, .. } = shader_input;
        ShaderMiracleSnowflakes::new(&shader_input).main_image(frag_color, frag_coord);
    }
}

const ITERATIONS: u32 = 15;
const DEPTH: f32 = 0.0125;
const LAYERS: f32 = 8.0;
const LAYERSBLOB: i32 = 20;
const STEP: f32 = 1.0;
const FAR: f32 = 10000.0;

pub struct ShaderMiracleSnowflakes<'a> {
    inputs: &'a ShaderInput,
    radius: f32,
    zoom: f32,

    light: Vec3,
    seed: Vec2,
    iteratorc: f32,
    powr: f32,
    res: f32,

    nray: Vec3,
    nray1: Vec3,
    nray2: Vec3,
    mxc: f32,
}

impl<'a> ShaderMiracleSnowflakes<'a> {
    #[must_use]
    fn new(inputs: &'a ShaderInput) -> Self {
        Self {
            inputs,
            radius: 0.25, // radius of Snowflakes. maximum for this demo 0.25.
            zoom: 4.0,    // use this to change details. optimal 0.1 - 4.0.
            light: vec3(0.0, 0.0, 1.0),
            seed: vec2(0.0, 0.0),
            iteratorc: ITERATIONS as f32,
            powr: 0.0,
            res: 0.0,

            nray: Vec3::ZERO,
            nray1: Vec3::ZERO,
            nray2: Vec3::ZERO,
            mxc: 1.0,
        }
    }
}

const NC0: Vec4 = vec4(0.0, 157.0, 113.0, 270.0);
const NC1: Vec4 = vec4(1.0, 158.0, 114.0, 271.0);

fn hash4(n: Vec4) -> Vec4 {
    (n.sin() * 1399763.5453123).fract_gl()
}
fn noise2(x: Vec2) -> f32 {
    let p = x.floor();
    let mut f = x.fract_gl();
    f = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let n = p.x + p.y * 157.0;
    let nc0 = NC0;
    let nc1 = NC1;
    let h = hash4(Vec4::splat(n) + vec4(nc0.x, nc0.y, nc1.x, nc1.y));
    let s1 = mix(h.xy(), h.zw(), f.xx());
    mix(s1.x, s1.y, f.y)
}

fn noise222(x: Vec2, y: Vec2, z: Vec2) -> f32 {
    let lx = vec4(x.x * y.x, x.y * y.x, x.x * y.y, x.y * y.y);
    let p = lx.floor();
    let mut f = lx.fract_gl();
    f = f * f * (Vec4::splat(3.0) - 2.0 * f);
    let n = p.xz() + p.yw() * 157.0;
    let h = mix(
        hash4(n.xxyy() + NC0.xyxy()),
        hash4(n.xxyy() + NC1.xyxy()),
        f.xxzz(),
    );
    mix(h.xz(), h.yw(), f.yw()).dot(z)
}

fn noise3(x: Vec3) -> f32 {
    let p = x.floor();
    let mut f = x.fract_gl();
    f = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let n = p.x + p.yz().dot(vec2(157.0, 113.0));
    let s1 = mix(
        hash4(Vec4::splat(n) + NC0),
        hash4(Vec4::splat(n) + NC1),
        f.xxxx(),
    );
    mix(mix(s1.x, s1.y, f.y), mix(s1.z, s1.w, f.y), f.z)
}
fn noise3_2(x: Vec3) -> Vec2 {
    vec2(noise3(x), noise3(x + Vec3::splat(100.0)))
}

impl<'a> ShaderMiracleSnowflakes<'a> {
    fn map(&self, rad: Vec2) -> f32 {
        let a;
        if self.res < 0.0015 {
            //a = noise2(rad.xy*20.6)*0.9+noise2(rad.xy*100.6)*0.1;
            a = noise222(rad, vec2(20.6, 100.6), vec2(0.9, 0.1));
        } else if self.res < 0.005 {
            //let a1: f32 = mix(noise2(rad.xy()*10.6),1.0,l);
            //a = texture(iChannel0,rad*0.3).x;
            a = noise2(rad * 20.6);
            //if a1<a {a=a1;}
        } else {
            a = noise2(rad * 10.3);
        }
        a - 0.5
    }

    fn dist_obj(&self, pos: Vec3, mut ray: Vec3, mut r: f32, seed: Vec2) -> Vec3 {
        let rq = r * r;
        let mut dist = ray * FAR;

        let norm = vec3(0.0, 0.0, 1.0);
        let invn = 1.0 / norm.dot(ray);
        let mut depthi = DEPTH;
        if invn < 0.0 {
            depthi = -depthi;
        }
        let mut ds = 2.0 * depthi * invn;
        let mut r1 = ray * (norm.dot(pos) - depthi) * invn - pos;
        let op1 = r1 + norm * depthi;
        let len1 = op1.dot(op1);
        let mut r2 = r1 + ray * ds;
        let op2 = r2 - norm * depthi;
        let len2 = op2.dot(op2);
        let n = ray.cross(norm).normalize();
        let mind = pos.dot(n);
        let n2 = ray.cross(n);
        let d = n2.dot(pos) / n2.dot(norm);
        let invd = 0.2 / DEPTH;

        if (len1 < rq || len2 < rq) || (mind.abs() < r && d <= DEPTH && d >= -DEPTH) {
            let _r3 = r2;
            let len = len1;
            if len >= rq {
                let n3 = norm.cross(n);
                let a = 1.0 / (rq - mind * mind).sqrt() * ray.dot(n3).abs();
                let dt = ray / a;
                r1 = -d * norm - mind * n - dt;
                if len2 >= rq {
                    r2 = -d * norm - mind * n + dt;
                }
                ds = (r2 - r1).dot(ray);
            }
            ds = (ds.abs() + 0.1) / (ITERATIONS as f32);
            ds = mix(DEPTH, ds, 0.2);
            if ds > 0.01 {
                ds = 0.01;
            }
            let ir = 0.35 / r;
            r *= self.zoom;
            ray = ray * ds * 5.0;
            for m in 0..ITERATIONS {
                if m as f32 >= self.iteratorc {
                    break;
                }
                let mut l = r1.xy().length(); //r1.xy().dot(r1.xy()).sqrt();
                let mut c3 = (r1.xy() / l).abs();
                if c3.x > 0.5 {
                    c3 = (c3 * 0.5 + vec2(-c3.y, c3.x) * 0.86602540).abs();
                }
                let g = l + c3.x * c3.x; //*1.047197551;
                l *= self.zoom;
                let mut h = l - r - 0.1;
                l = l.powf(self.powr) + 0.1;
                h = h.max(mix(self.map(c3 * l + seed), 1.0, (r1.z * invd).abs())) + g * ir - 0.245; //0.7*0.35=0.245 //*0.911890636
                if (h < self.res * 20.0) || r1.z.abs() > DEPTH + 0.01 {
                    break;
                }
                r1 += ray * h;
                ray *= 0.99;
            }
            if r1.z.abs() < DEPTH + 0.01 {
                dist = r1 + pos;
            }
        }
        dist
    }

    fn filter_flake(
        &mut self,
        mut color: Vec4,
        pos: Vec3,
        ray: Vec3,
        ray1: Vec3,
        ray2: Vec3,
    ) -> Vec4 {
        let d = self.dist_obj(pos, ray, self.radius, self.seed);
        let n1 = self.dist_obj(pos, ray1, self.radius, self.seed);
        let n2 = self.dist_obj(pos, ray2, self.radius, self.seed);

        let lq = vec3(d.dot(d), n1.dot(n1), n2.dot(n2));
        if lq.x < FAR || lq.y < FAR || lq.z < FAR {
            let n = (n1 - d).cross(n2 - d).normalize();
            if lq.x < FAR && lq.y < FAR && lq.z < FAR {
                self.nray = n; //(self.nray+n).normalize();
                               //self.nray1 = (ray1+n).normalize();
                               //self.nray2 = (ray2+n).normalize();
            }
            let da = n.dot(self.light).abs().powf(3.0);
            let mut cf = mix(vec3(0.0, 0.4, 1.0), color.xyz() * 10.0, n.dot(ray).abs());
            cf = mix(cf, Vec3::splat(2.0), da);
            color = (mix(
                color.xyz(),
                cf,
                self.mxc * self.mxc * (0.5 + n.dot(ray).abs() * 0.5),
            ))
            .extend(color.w);
        }

        color
    }

    fn main_image(&mut self, frag_color: &mut Vec4, frag_coord: Vec2) {
        let time = self.inputs.time * 0.2; //*0.1;
        self.res = 1.0 / self.inputs.resolution.y;
        let p = (-self.inputs.resolution.xy() + 2.0 * frag_coord) * self.res;

        let mut rotate;
        let mut mr;
        let mut ray;
        let mut ray1;
        let mut ray2;
        let mut pos = vec3(0.0, 0.0, 1.0);

        *frag_color = vec4(0.0, 0.0, 0.0, 0.0);
        self.nray = Vec3::ZERO;
        self.nray1 = Vec3::ZERO;
        self.nray2 = Vec3::ZERO;

        let mut refcolor: Vec4 = Vec4::ZERO;
        self.iteratorc = ITERATIONS as f32 - LAYERS;

        let mut addrot = Vec2::ZERO;
        if self.inputs.mouse.z > 0.0 {
            addrot = (self.inputs.mouse.xy() - self.inputs.resolution.xy() * 0.5) * self.res;
        }

        let mut mxcl = 1.0;
        let mut addpos = Vec3::ZERO;
        pos.z = 1.0;
        self.mxc = 1.0;
        self.radius = 0.25;
        let mzd: f32 = (self.zoom - 0.1) / LAYERS;
        for i in 0..LAYERSBLOB {
            let p2 = p - Vec2::splat(0.25) + Vec2::splat(0.1 * i as f32);
            ray = p2.extend(2.0) - self.nray * 2.0;
            //ray = self.nray;//*0.6;
            ray1 = (ray + vec3(0.0, self.res * 2.0, 0.0)).normalize();
            ray2 = (ray + vec3(self.res * 2.0, 0.0, 0.0)).normalize();
            ray = ray.normalize();
            let mut sb = ray.xy() * pos.length() / pos.normalize().dot(ray) + vec2(0.0, time);
            self.seed = (sb + vec2(0.0, pos.z)).floor() + Vec2::splat(pos.z);
            let mut seedn = self.seed.extend(pos.z);
            sb = sb.floor();
            if noise3(seedn) > 0.2 && i < LAYERS as i32 {
                self.powr = noise3(seedn * 10.0) * 1.9 + 0.1;
                rotate = (((Vec2::splat(0.5) - noise3_2(seedn)) * time * 5.0).sin() * 0.3 + addrot)
                    .extend(0.0);
                rotate.z = (0.5 - noise3(seedn + vec3(10.0, 3.0, 1.0))) * time * 5.0;
                seedn.z += time * 0.5;
                addpos = (sb + vec2(0.25, 0.25 - time) + noise3_2(seedn) * 0.5).extend(addpos.z);
                let sins = rotate.sin();
                let coss = rotate.cos();
                mr = Mat3::from_cols(
                    vec3(coss.x, 0.0, sins.x),
                    vec3(0.0, 1.0, 0.0),
                    vec3(-sins.x, 0.0, coss.x),
                );
                mr = Mat3::from_cols(
                    vec3(1.0, 0.0, 0.0),
                    vec3(0.0, coss.y, sins.y),
                    vec3(0.0, -sins.y, coss.y),
                ) * mr;
                mr = Mat3::from_cols(
                    vec3(coss.z, sins.z, 0.0),
                    vec3(-sins.z, coss.z, 0.0),
                    vec3(0.0, 0.0, 1.0),
                ) * mr;

                self.light = mr.transpose() * vec3(1.0, 0.0, 1.0).normalize();
                // let cc = self.filter_flake(
                // *frag_color,
                // mr.transpose() * (pos + addpos),
                // (mr.transpose() * ray + self.nray * 0.1).normalize(),
                // (mr.transpose() * ray1 + self.nray * 0.1).normalize(),
                // (mr.transpose() * ray2 + self.nray * 0.1).normalize(),
                // );
                let mut cc = self.filter_flake(
                    *frag_color,
                    mr.transpose() * (pos + addpos),
                    mr.transpose() * ray,
                    mr.transpose() * ray1,
                    mr.transpose() * ray2,
                );
                if false {
                    if i > 0
                        && self.nray.dot(self.nray) != 0.0
                        && self.nray1.dot(self.nray1) != 0.0
                        && self.nray2.dot(self.nray2) != 0.0
                    {
                        refcolor = self.filter_flake(
                            refcolor,
                            mr.transpose() * (pos + addpos),
                            self.nray,
                            self.nray1,
                            self.nray2,
                        );
                    }
                    cc += refcolor * 0.5;
                }
                *frag_color = mix(cc, *frag_color, frag_color.w.min(1.0));
            }
            seedn = sb.extend(pos.z) + vec3(0.5, 1000.0, 300.0);
            if noise3(seedn * 10.0) > 0.4 {
                let raf = 0.3 + noise3(seedn * 100.0);
                addpos =
                    (sb + vec2(0.2, 0.2 - time) + noise3_2(seedn * 100.0) * 0.6).extend(addpos.z);
                let mut l = (ray * ray.dot(pos + addpos) - pos - addpos).length();
                l = (1.0 - l * 10.0 * raf).max(0.0);
                *frag_color +=
                    vec4(1.0, 1.2, 3.0, 1.0) * l.powf(5.0) * ((0.6 + raf).powf(2.0) - 0.6) * mxcl;
            }
            self.mxc -= 1.1 / LAYERS;
            pos.z += STEP;
            self.iteratorc += 2.0;
            mxcl -= 1.1 / LAYERSBLOB as f32;
            self.zoom -= mzd;
        }

        let cr = mix(Vec3::ZERO, vec3(0.0, 0.0, 0.4), (-0.55 + p.y) * 2.0);
        *frag_color = (frag_color.xyz()
            + mix(
                (cr - frag_color.xyz()) * 0.1,
                vec3(0.2, 0.5, 1.0),
                ((-p.y + 1.0) * 0.5).clamp(0.0, 1.0),
            ))
        .extend(frag_color.z);

        *frag_color = Vec4::ONE.min(*frag_color);
    }
}
