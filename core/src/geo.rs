use bevy::math::*;
use std::cmp::PartialOrd;
use std::ops::{Add, Mul, Sub};

pub type Bounds1D = Bounds<f32>;
pub type Bounds2D = Bounds<Vec2>;
pub type Bounds3D = Bounds<Vec3>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Bounds<T> {
    pub center: T,
    pub extents: T,
}

impl<T: Copy> Bounds<T> {
    pub fn new(center: T, extents: T) -> Self {
        Self { center, extents }
    }
}

impl<T: Add<Output = T> + Clone> Bounds<T> {
    /// Gets the highest value in the bounds.
    pub fn max(&self) -> <T as Add>::Output {
        self.center.clone() + self.extents.clone()
    }

    pub fn expand(&mut self, size: T) {
        self.extents = self.extents.clone() + size.clone();
    }
}

impl<T: Sub<Output = T> + Clone> Bounds<T> {
    pub fn min(&self) -> <T as Sub>::Output {
        self.center.clone() - self.extents.clone()
    }

    pub fn shrink(&mut self, size: T) {
        self.extents = self.extents.clone() - size.clone();
    }
}

impl<T: Mul<f32> + Clone> Bounds<T> {
    /// Gets the full size of the bounds.
    pub fn size(&self) -> <T as Mul<f32>>::Output {
        self.extents.clone() * 2.0
    }
}

impl<T: Add<Output = T> + Sub<Output = T> + PartialOrd + Clone> Bounds<T> {
    /// Checks if the range contains the point.
    pub fn contains_point(&self, check: T) -> bool {
        self.max() >= check && self.min() <= check
    }

    /// Checks if the target bounds is entirely contained within the current bound.
    pub fn contains_bounds(&self, other: Self) -> bool {
        self.max() >= other.max() && self.min() <= other.min()
    }

    /// Checks if two bounds intersect.
    pub fn intersects(&self, other: &Self) -> bool {
        let combo = self.extents.clone() + other.extents.clone();
        let diff = if self.center > other.center {
            self.center.clone() - other.center.clone()
        } else {
            other.center.clone() - self.center.clone()
        };
        combo >= diff
    }
}

impl<T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy + PartialOrd> Bounds<T> {
    /// Gets the full size of the bounds.
    pub fn from_min_max(min: T, max: T) -> Self {
        let center = (min + max) * 0.5;
        let size = if min < max { max - min } else { min - max };
        Self::new(center, size * 0.5)
    }
}

impl Bounds1D {
    pub fn encapsulate(&mut self, value: f32) {
        let max = self.max();
        let min = self.min();
        let delta = if max < value {
            (value - max) / 2.0
        } else if min > value {
            (min - value) / 2.0
        } else {
            return;
        };
        self.center += delta;
        self.extents += delta;
    }
}

impl Bounds2D {
    pub fn encapsulate(&mut self, value: Vec2) {
        let (mut x, mut y) = self.decompose();
        x.encapsulate(value.x);
        y.encapsulate(value.y);
        *self = Self::from((x, y));
    }

    pub fn decompose(self) -> (Bounds1D, Bounds1D) {
        (
            Bounds1D::new(self.center.x, self.extents.x),
            Bounds1D::new(self.center.y, self.extents.y),
        )
    }

    /// Merges another bounds into the current one.
    pub fn merge_with(&mut self, other: Self) {
        let (mut min_x, mut min_y) = self.min().into();
        let (mut max_x, mut max_y) = self.max().into();
        let tests = [other.max(), other.min()];
        for test in tests.iter() {
            if test.x > max_x {
                max_x = test.x;
            }
            if test.x < min_x {
                min_x = test.x;
            }
            if test.y > max_y {
                max_y = test.y;
            }
            if test.y < min_y {
                min_y = test.y;
            }
        }
        *self = Self::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
    }
}

impl From<(Bounds1D, Bounds1D)> for Bounds2D {
    fn from(value: (Bounds1D, Bounds1D)) -> Self {
        Self {
            center: Vec2::new(value.0.center, value.1.center),
            extents: Vec2::new(value.0.extents, value.1.extents),
        }
    }
}

impl From<Bounds2D> for Rect<f32> {
    fn from(value: Bounds2D) -> Self {
        let min = value.min();
        let max = value.max();
        Self {
            left: min.x,
            right: max.x,
            top: max.y,
            bottom: min.y,
        }
    }
}

impl From<Rect<f32>> for Bounds2D {
    fn from(value: Rect<f32>) -> Self {
        Self::from((
            Bounds1D::from_min_max(value.left, value.right),
            Bounds1D::from_min_max(value.top, value.bottom),
        ))
    }
}

impl Bounds3D {
    pub fn encapsulate(&mut self, value: Vec3) {
        let (mut x, mut y, mut z) = self.decompose();
        x.encapsulate(value.x);
        y.encapsulate(value.y);
        z.encapsulate(value.z);
        *self = Self::from((x, y, z));
    }

    pub fn decompose(self) -> (Bounds1D, Bounds1D, Bounds1D) {
        (
            Bounds1D::new(self.center.x, self.extents.x),
            Bounds1D::new(self.center.y, self.extents.y),
            Bounds1D::new(self.center.z, self.extents.z),
        )
    }

    /// Merges another bounds into the current one.
    pub fn merge_with(&mut self, other: Self) {
        let (mut min_x, mut min_y, mut min_z) = self.min().into();
        let (mut max_x, mut max_y, mut max_z) = self.max().into();
        let tests = [other.max(), other.min()];
        for test in tests.iter() {
            if test.x > max_x {
                max_x = test.x;
            }
            if test.x < min_x {
                min_x = test.x;
            }
            if test.y > max_y {
                max_y = test.y;
            }
            if test.y < min_y {
                min_y = test.y;
            }
            if test.z > max_z {
                max_z = test.z;
            }
            if test.z < min_z {
                min_z = test.z;
            }
        }
        *self = Self::from_min_max(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        );
    }
}

impl From<(Bounds1D, Bounds1D, Bounds1D)> for Bounds3D {
    fn from(value: (Bounds1D, Bounds1D, Bounds1D)) -> Self {
        Self {
            center: Vec3::new(value.0.center, value.1.center, value.2.center),
            extents: Vec3::new(value.0.extents, value.1.extents, value.2.extents),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_bounds_1d_contains_point() {
        let bounds = Bounds1D::new(1.0, 3.0);

        assert!(bounds.contains_point(3.0));
        assert!(bounds.contains_point(-1.0));
        assert!(bounds.contains_point(4.0));
        assert!(bounds.contains_point(-2.0));
        assert!(!bounds.contains_point(5.0));
        assert!(!bounds.contains_point(-3.0));
    }

    #[test]
    pub fn test_bounds_1d_max() {
        let bounds = Bounds1D::new(1.0, 3.0);
        assert_eq!(bounds.max(), 4.0);
    }

    #[test]
    pub fn test_bounds_1d_min() {
        let bounds = Bounds1D::new(1.0, 3.0);
        assert_eq!(bounds.min(), -2.0);
    }

    #[test]
    pub fn test_bounds_1d_expand() {
        let mut bounds = Bounds1D::new(1.0, 3.0);
        bounds.expand(2.0);
        assert_eq!(bounds, Bounds1D::new(1.0, 5.0));
    }

    #[test]
    pub fn test_bounds_1d_shrink() {
        let mut bounds = Bounds1D::new(1.0, 3.0);
        bounds.shrink(2.0);
        assert_eq!(bounds, Bounds1D::new(1.0, 1.0));
    }

    #[test]
    pub fn test_bounds_1d_size() {
        let bounds = Bounds1D::new(1.0, 3.0);
        assert_eq!(bounds.size(), 6.0);
    }

    #[test]
    pub fn test_bounds_1d_from_min_max() {
        let bounds = Bounds1D::from_min_max(1.0, 3.0);
        assert_eq!(bounds.max(), 3.0);
        assert_eq!(bounds.min(), 1.0);
    }

    #[test]
    pub fn test_bounds_1d_intersects() {
        let bounds = Bounds1D::new(1.0, 3.0);
        let a = Bounds1D::new(-6.0, 3.0);
        let b = Bounds1D::new(-3.0, 3.0);
        assert!(!bounds.intersects(&a));
        assert!(bounds.intersects(&b));
        assert!(!a.intersects(&bounds));
        assert!(b.intersects(&bounds));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    pub fn test_bounds_1d_encapsulates() {
        let mut bounds = Bounds1D::new(1.0, 3.0);
        bounds.encapsulate(5.0);
        assert_eq!(bounds, Bounds1D::new(1.5, 3.5));
    }
}
