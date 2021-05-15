use super::{point::Point, Bounds};
use bevy::math::*;

pub type LineSegment2D = LineSegment<Vec2>;
pub type LineSegment3D = LineSegment<Vec3>;
pub type LineSegment4D = LineSegment<Vec4>;

#[derive(PartialEq, Eq)]
enum TripletOrientation {
    Colinear,
    Clockwise,
    CounterClockwise,
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct LineSegment<T> {
    pub start: T,
    pub end: T,
}

impl<T: Point> LineSegment<T> {
    pub fn new(start: impl Into<T>, end: impl Into<T>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn length(&self) -> f32 {
        self.diff().length()
    }

    pub fn length_squared(&self) -> f32 {
        self.diff().length_squared()
    }

    pub fn bounds(&self) -> Bounds<T> {
        Bounds::<T> {
            center: self.end * 0.5 + self.start * 0.5,
            extents: (self.start - self.end).abs() * 0.5,
        }
    }

    #[inline(always)]
    fn diff(&self) -> T {
        self.end - self.start
    }
}

impl LineSegment2D {
    pub fn intersects(&self, other: Self) -> bool {
        let o1 = Self::triplet_orientation(self.start, self.end, other.start);
        let o2 = Self::triplet_orientation(self.start, self.end, other.end);
        let o3 = Self::triplet_orientation(other.start, other.end, self.start);
        let o4 = Self::triplet_orientation(other.start, other.end, self.end);

        let c1 = Self::point_on_segment(self.start, other.end, self.end);
        let c2 = Self::point_on_segment(other.start, self.end, other.end);

        // General case
        (o1 != o2 && o3 != o4) ||
        // colinear cases
        (o1 == TripletOrientation::Colinear && c1) ||
        (o2 == TripletOrientation::Colinear && c1) ||
        (o3 == TripletOrientation::Colinear && c2) ||
        (o4 == TripletOrientation::Colinear && c2)
    }

    fn triplet_orientation(p: Vec2, q: Vec2, r: Vec2) -> TripletOrientation {
        match (q - p).yx().dot(r - q) {
            x if x > 0.0 => TripletOrientation::Clockwise,
            x if x < 0.0 => TripletOrientation::CounterClockwise,
            _ => TripletOrientation::Colinear,
        }
    }

    /// Given p, q, r are colinear,
    /// checks if point q lies on segment pr
    fn point_on_segment(p: Vec2, q: Vec2, r: Vec2) -> bool {
        q.cmple(p.max(r)).all() && q.cmpge(p.min(r)).all()
    }

    pub fn world_position(&self, x: f32) -> Vec2 {
        // TODO(james7123): This will produce a invalid result if the platform is vertical,
        // ensure this doesn't happen.
        let diff = self.diff();
        let y = (diff.y / diff.x) * (x - self.start.x) + self.start.y;
        (x, y).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_colinear_intersection() {
        let a = LineSegment2D::new((0.0, 0.0), (10.0, 10.0));
        let b = LineSegment2D::new((15.0, 15.0), (20.0, 20.0));
        let c = LineSegment2D::new((9.0, 9.0), (16.0, 16.0));
        assert!(!a.intersects(b));
        assert!(!b.intersects(a));
        assert!(c.intersects(b));
        assert!(c.intersects(a));
        assert!(a.intersects(c));
        assert!(b.intersects(c));
    }
}
