use bevy::math::*;
use std::ops::{Add, Mul, Sub};

pub trait Point: Add<Output = Self> + Sub<Output = Self> + Mul<f32, Output = Self> + Copy {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn abs(self) -> Self;
    fn cmple(self, other: Self) -> bool;
    fn lerp(self, other: Self, x: f32) -> Self;
    fn dot(self, other: Self) -> f32;

    #[inline(always)]
    fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline(always)]
    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
}

impl Point for f32 {
    fn min(self, other: Self) -> Self {
        f32::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }

    fn abs(self) -> Self {
        f32::abs(self)
    }

    fn cmple(self, other: Self) -> bool {
        self <= other
    }

    fn lerp(self, other: Self, x: f32) -> Self {
        self * (1.0 - x) + other * x
    }

    fn dot(self, other: Self) -> Self {
        self * other
    }
}

macro_rules! boundable {
    ($type:ty) => {
        impl Point for $type {
            #[inline(always)]
            fn min(self, other: Self) -> Self {
                <$type>::min(self, other)
            }

            #[inline(always)]
            fn max(self, other: Self) -> Self {
                <$type>::max(self, other)
            }

            #[inline(always)]
            fn abs(self) -> Self {
                <$type>::abs(self)
            }

            #[inline(always)]
            fn cmple(self, other: Self) -> bool {
                <$type>::cmple(self, other).all()
            }

            #[inline(always)]
            fn lerp(self, other: Self, x: f32) -> Self {
                <$type>::lerp(self, other, x)
            }

            #[inline(always)]
            fn dot(self, other: Self) -> f32 {
                <$type>::dot(self, other)
            }
        }
    };
}

boundable!(Vec2);
boundable!(Vec3);
boundable!(Vec4);
