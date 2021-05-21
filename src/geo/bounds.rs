use super::point::Point;
use bevy::math::*;

pub type Bounds1D = Bounds<f32>;
pub type Bounds2D = Bounds<Vec2>;
pub type Bounds3D = Bounds<Vec3>;
pub type Bounds4D = Bounds<Vec4>;

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Bounds<T> {
    pub center: T,
    pub extents: T,
}

impl<T: Point> Bounds<T> {
    pub fn new(center: T, extents: T) -> Self {
        Self {
            center: center,
            extents: extents,
        }
    }

    /// Gets the highest value in the bounds.
    pub fn max(&self) -> T {
        self.center + self.extents
    }

    pub fn min(&self) -> T {
        self.center - self.extents
    }

    pub fn set_min_max(&mut self, min: T, max: T) {
        self.center = (min + max) * 0.5;
        self.extents = (max - self.center).abs();
    }

    pub fn encapsulate(&mut self, value: T) {
        self.set_min_max(self.min().min(value), self.max().max(value));
    }

    /// Merges another bounds into the current one.
    pub fn merge_with(&mut self, other: Self) {
        self.set_min_max(self.min().min(other.min()), self.max().max(other.max()))
    }

    pub fn expand(&mut self, size: T) {
        self.extents = self.extents + size;
    }

    pub fn shrink(&mut self, size: T) {
        self.extents = self.extents - size;
    }

    /// Gets the full size of the bounds.
    pub fn size(&self) -> T {
        self.extents * 2.0
    }

    /// Gets the full size of the bounds.
    pub fn from_min_max(min: T, max: T) -> Self {
        let center = (min + max) * 0.5;
        let size = (max - center).abs();
        Self::new(center, size)
    }

    /// Checks if the range contains the point.
    pub fn contains_point(&self, check: T) -> bool {
        (self.center - check).abs().cmple(self.extents)
    }

    /// Checks if the target bounds is entirely contained within the current bound.
    pub fn contains_bounds(&self, other: Self) -> bool {
        self.max().cmple(other.max()) && other.min().cmple(self.min())
    }

    /// Checks if two bounds intersect.
    /// Returns true if either is entirely inside of the other.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min().cmple(other.max()) && other.min().cmple(self.max())
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
