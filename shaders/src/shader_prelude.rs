pub use core::f32::consts::{FRAC_1_PI, FRAC_PI_2, PI, TAU};
use core::ops::{Add, Mul, Sub};
/// We can't use the `f32::consts::SQRT_3` constant here because it is an unstable library feature
pub const SQRT_3: f32 = 1.732_050_807_568_877_2;

pub use crate::shared_data::ShaderConstants;
pub use spirv_std::{
    arch::Derivative,
    glam::{
        mat2, mat3, vec2, vec3, vec4, Mat2, Mat3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4,
        Vec4Swizzles,
    },
    spirv,
};

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
pub use spirv_std::num_traits::Float;

pub trait SampleCube: Copy {
    fn sample_cube(self, p: Vec3) -> Vec4;
}

#[derive(Copy, Clone)]
pub struct ConstantColor {
    pub color: Vec4,
}

impl SampleCube for ConstantColor {
    fn sample_cube(self, _: Vec3) -> Vec4 {
        self.color
    }
}

#[derive(Copy, Clone)]
pub struct RgbCube {
    pub alpha: f32,
    pub intensity: f32,
}

impl SampleCube for RgbCube {
    fn sample_cube(self, p: Vec3) -> Vec4 {
        (p.abs() * self.intensity).extend(self.alpha)
    }
}

pub struct ShaderInput {
    pub resolution: Vec3,
    pub time: f32,
    pub frag_coord: Vec2,
    /// https://www.shadertoy.com/view/Mss3zH
    pub mouse: Vec4,
}

pub struct ShaderResult {
    pub color: Vec4,
}

pub struct ShaderDefinition {
    pub name: &'static str,
}

#[inline(always)]
#[must_use]
pub fn saturate_vec3(a: Vec3) -> Vec3 {
    a.clamp(Vec3::ZERO, Vec3::ONE)
}
#[inline(always)]
#[must_use]
pub fn saturate_vec2(a: Vec2) -> Vec2 {
    a.clamp(Vec2::ZERO, Vec2::ONE)
}
#[inline(always)]
#[must_use]
pub fn saturate(a: f32) -> f32 {
    a.clamp(0.0, 1.0)
}

/// Based on: https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/
#[inline]
#[must_use]
pub fn acos_approx(v: f32) -> f32 {
    let x = v.abs();
    let mut res = (-0.155_972_f32).mul_add(x, 1.56467); // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0 {
        res
    } else {
        PI - res
    }
}

#[inline(always)]
#[must_use]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * 2.0f32.mul_add(-x, 3.0)
}

#[inline(always)]
#[must_use]
pub fn mix<X: Copy + Mul<A, Output = X> + Add<Output = X> + Sub<Output = X>, A: Copy>(
    x: X,
    y: X,
    a: A,
) -> X {
    x - x * a + y * a
}

pub trait Clamp {
    #[must_use]
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Clamp for f32 {
    #[inline(always)]
    fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}

pub trait FloatExt {
    #[must_use]
    fn fract_gl(self) -> Self;
    #[must_use]
    fn rem_euclid(self, rhs: Self) -> Self;
    #[must_use]
    fn sign_gl(self) -> Self;
    #[must_use]
    fn step(self, x: Self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn fract_gl(self) -> Self {
        self - self.floor()
    }

    #[inline]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
    }

    #[inline]
    fn sign_gl(self) -> Self {
        if self < 0.0 {
            -1.0
        } else if self == 0.0 {
            0.0
        } else {
            1.0
        }
    }

    #[inline]
    fn step(self, x: Self) -> Self {
        if x < self {
            0.0
        } else {
            1.0
        }
    }
}

pub trait VecExt {
    #[must_use]
    fn sin(self) -> Self;
    #[must_use]
    fn cos(self) -> Self;
    #[must_use]
    fn powf_vec(self, p: Self) -> Self;
    #[must_use]
    fn sqrt(self) -> Self;
    #[must_use]
    fn ln(self) -> Self;
    #[must_use]
    fn step(self, other: Self) -> Self;
    #[must_use]
    fn sign_gl(self) -> Self;
}

impl VecExt for Vec2 {
    #[inline]
    fn sin(self) -> Self {
        vec2(self.x.sin(), self.y.sin())
    }

    #[inline]
    fn cos(self) -> Self {
        vec2(self.x.cos(), self.y.cos())
    }

    #[inline]
    fn powf_vec(self, p: Self) -> Self {
        vec2(self.x.powf(p.x), self.y.powf(p.y))
    }

    #[inline]
    fn sqrt(self) -> Self {
        vec2(self.x.sqrt(), self.y.sqrt())
    }

    #[inline]
    fn ln(self) -> Self {
        vec2(self.x.ln(), self.y.ln())
    }

    #[inline]
    fn step(self, other: Self) -> Self {
        vec2(self.x.step(other.x), self.y.step(other.y))
    }

    #[inline]
    fn sign_gl(self) -> Self {
        vec2(self.x.sign_gl(), self.y.sign_gl())
    }
}

impl VecExt for Vec3 {
    #[inline]
    fn sin(self) -> Self {
        vec3(self.x.sin(), self.y.sin(), self.z.sin())
    }

    #[inline]
    fn cos(self) -> Self {
        vec3(self.x.cos(), self.y.cos(), self.z.cos())
    }

    #[inline]
    fn powf_vec(self, p: Self) -> Self {
        vec3(self.x.powf(p.x), self.y.powf(p.y), self.z.powf(p.z))
    }

    #[inline]
    fn sqrt(self) -> Self {
        vec3(self.x.sqrt(), self.y.sqrt(), self.z.sqrt())
    }

    #[inline]
    fn ln(self) -> Self {
        vec3(self.x.ln(), self.y.ln(), self.z.ln())
    }

    #[inline]
    fn step(self, other: Self) -> Self {
        vec3(
            self.x.step(other.x),
            self.y.step(other.y),
            self.z.step(other.z),
        )
    }

    #[inline]
    fn sign_gl(self) -> Self {
        vec3(self.x.sign_gl(), self.y.sign_gl(), self.z.sign_gl())
    }
}

impl VecExt for Vec4 {
    #[inline]
    fn sin(self) -> Self {
        vec4(self.x.sin(), self.y.sin(), self.z.sin(), self.w.sin())
    }

    #[inline]
    fn cos(self) -> Self {
        vec4(self.x.cos(), self.y.cos(), self.z.cos(), self.w.cos())
    }

    #[inline]
    fn powf_vec(self, p: Self) -> Self {
        vec4(
            self.x.powf(p.x),
            self.y.powf(p.y),
            self.z.powf(p.z),
            self.w.powf(p.w),
        )
    }

    #[inline]
    fn sqrt(self) -> Self {
        vec4(self.x.sqrt(), self.y.sqrt(), self.z.sqrt(), self.w.sqrt())
    }

    #[inline]
    fn ln(self) -> Self {
        vec4(self.x.ln(), self.y.ln(), self.z.ln(), self.w.ln())
    }

    #[inline]
    fn step(self, other: Self) -> Self {
        vec4(
            self.x.step(other.x),
            self.y.step(other.y),
            self.z.step(other.z),
            self.w.step(other.w),
        )
    }

    #[inline]
    fn sign_gl(self) -> Self {
        vec4(
            self.x.sign_gl(),
            self.y.sign_gl(),
            self.z.sign_gl(),
            self.w.sign_gl(),
        )
    }
}

#[inline(always)]
pub fn discard() {
    unsafe { spirv_std::arch::demote_to_helper_invocation() }
}
