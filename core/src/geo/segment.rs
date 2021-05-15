use super::{point::Point, Bounds};
use bevy::math::*;

pub type LineSegment2D = LineSegment<Vec2>;
pub type LineSegment3D = LineSegment<Vec3>;
pub type LineSegment4D = LineSegment<Vec4>;

#[derive(Debug, PartialEq, Eq)]
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

        let c1 = Self::point_on_segment(self.start, self.end, other.end);
        let c2 = Self::point_on_segment(other.start, other.end, self.end);

        // General case
        (o1 != o2 && o3 != o4) ||
        // colinear cases
        (o1 == TripletOrientation::Colinear && c1) ||
        (o2 == TripletOrientation::Colinear && c1) ||
        (o3 == TripletOrientation::Colinear && c2) ||
        (o4 == TripletOrientation::Colinear && c2)
    }

    fn triplet_orientation(p: Vec2, q: Vec2, r: Vec2) -> TripletOrientation {
        let qp = q - p;
        let rq = r - q;
        match qp.y * rq.x - qp.x * rq.y {
            x if x > 0.0 => TripletOrientation::Clockwise,
            x if x < 0.0 => TripletOrientation::CounterClockwise,
            _ => TripletOrientation::Colinear,
        }
    }

    /// Given p, q, r are colinear,
    /// checks if point q lies on segment pr
    fn point_on_segment(p: Vec2, q: Vec2, r: Vec2) -> bool {
        let (x, y) = ((r - p) / (q - p)).into();
        x == y && x >= 0.0 && x <= 1.0
    }

    pub fn world_position(&self, x: f32) -> Vec2 {
        // TODO(james7123): This will produce a invalid result if the platform is vertical,
        // ensure this doesn't happen.
        let diff = self.diff();
        let y = (diff.y / diff.x) * (x - self.start.x) + self.start.y;
        (x, y).into()
    }
}

impl LineSegment3D {
    pub fn sqr_distance(&self, other: Self) -> f32 {
        const SMALL_NUM: f32 = 0.00000001;
        let u = self.end - self.start;
        let v = other.end - other.start;
        let w = self.start - other.start;
        let a = u.dot(u);
        let b = u.dot(v);
        let c = v.dot(v);
        let d = u.dot(w);
        let e = v.dot(w);
        let D = a * c - b * b;

        let (mut sN, mut sD) = (D, D); // sc = sN/sD, default sD=D >= 0
        let (mut tN, mut tD) = (D, D); // tc = tN/tD, default tD=D >= 0

        // compute the line parameters of the two closest points
        if D < SMALL_NUM {
            // the lines are almost parallel
            sN = 0.0; // force using point P0 on segment S1
            sD = 1.0; // to prevent possible division by 0 later
            tN = e;
            tD = c;
        } else {
            // get the closest points on the infinite lines
            sN = b * e - c * d;
            tN = a * e - b * d;
            if sN < 0.0 {
                // sc < 0 => the s=0 edge is visible
                sN = 0.0;
                tN = e;
                tD = c;
            } else if sN > sD {
                // sc > 1  => the s=1 edge is visible
                sN = sD;
                tN = e + b;
                tD = c;
            }
        }

        if tN < 0.0 {
            // tc < 0 => the t=0 edge is visible
            tN = 0.0;
            // recompute sc for this edge
            if -d < 0.0 {
                sN = 0.0;
            } else if -d > a {
                sN = sD;
            } else {
                sN = -d;
                sD = a;
            }
        } else if tN > tD {
            // tc > 1  => the t=1 edge is visible
            tN = tD;
            // recompute sc for this edge
            if -d + b < 0.0 {
                sN = 0.0;
            } else if -d + b > a {
                sN = sD;
            } else {
                sN = -d + b;
                sD = a;
            }
        }
        // finally do the division to get sc and tc
        let sc = if sN.abs() < SMALL_NUM { 0.0 } else { sN / sD };
        let tc = if tN.abs() < SMALL_NUM { 0.0 } else { tN / tD };

        // get the difference of the two closest points
        let dp = w + (sc * u) - (tc * v); // =  S1(sc) - S2(tc)
        dp.dot(dp)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_intersection() {
        let a = LineSegment2D::new((-1.0, 0.0), (1.0, 0.0));
        let b = LineSegment2D::new((0.0, -1.0), (0.0, 1.0));
        assert!(a.intersects(b));
        assert!(b.intersects(a));
    }

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

    #[test]
    fn test_sqr_distance_simple() {
        let a = LineSegment3D::new((-3.0, 0.0, 0.0), (3.0, 0.0, 0.0));
        let b = LineSegment3D::new((10.0, -3.0, 0.0), (10.0, 3.0, 0.0));
        assert_eq!(a.sqr_distance(b), 49.0);
    }

    #[test]
    fn test_sqr_distance_intersection() {
        let a = LineSegment3D::new((-3.0, 0.0, 0.0), (3.0, 0.0, 0.0));
        let b = LineSegment3D::new((0.0, -3.0, 0.0), (0.0, 3.0, 0.0));
        let c = LineSegment3D::new((0.0, 0.0, -3.0), (0.0, 0.0, 3.0));
        assert_eq!(a.sqr_distance(b), 0.0);
        assert_eq!(a.sqr_distance(c), 0.0);
        assert_eq!(b.sqr_distance(a), 0.0);
        assert_eq!(b.sqr_distance(c), 0.0);
        assert_eq!(c.sqr_distance(a), 0.0);
        assert_eq!(c.sqr_distance(b), 0.0);
    }
}
