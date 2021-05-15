use super::segment::LineSegment3D;
use bevy::math::*;

pub type Capsule3D = Capsule<Vec3>;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Capsule<T> {
    pub start: T,
    pub end: T,
    pub radius: f32,
}

impl Capsule3D {
    pub fn sphere(center: impl Into<Vec3>, radius: f32) -> Self {
        let center = center.into();
        Self {
            start: center,
            end: center,
            radius,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        // shortcutting sphere-sphere collisions
        let sqr_distance = if self.is_sphere() && other.is_sphere() {
            let diff = self.start - other.start;
            diff.dot(diff)
        } else {
            self.segment().sqr_distance(other.segment())
        };
        let dist = self.radius + other.radius;
        sqr_distance <= dist * dist
    }

    fn is_sphere(&self) -> bool {
        self.start == self.end
    }

    fn segment(&self) -> LineSegment3D {
        LineSegment3D {
            start: self.start,
            end: self.end,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_intersects_simple() {
        let a = Capsule3D {
            start: (-3.0, 0.0, 0.0).into(),
            end: (3.0, 0.0, 0.0).into(),
            radius: 4.0,
        };
        let b = Capsule3D {
            start: (10.0, -3.0, 0.0).into(),
            end: (3.0, 0.0, 0.0).into(),
            radius: 3.0,
        };
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_intersects_sphere_sphere() {
        let a = Capsule3D::sphere((0.0, 0.0, 0.0), 3.0);
        let b = Capsule3D::sphere((11.0, 0.0, 0.0), 8.0);
        assert!(a.intersects(&b));
    }
}
