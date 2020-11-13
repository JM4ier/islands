#![allow(unused)]
pub type Float = f32;
pub type Vector3 = nalgebra::base::Vector3<Float>;
pub type Vector2 = nalgebra::base::Vector2<Float>;

#[derive(Debug, Clone)]
pub struct Triangle(pub [Vector3; 3]);

impl Triangle {
    #[allow(unused)]
    /// Normal vector of the triangle pointing perpendicular to its surface
    pub fn normal(&self) -> Vector3 {
        let vec = self.0;
        (vec[1] - vec[0]).cross(&(vec[2] - vec[0])).normalize()
    }
}

#[derive(Debug, Clone)]
pub struct Line2 {
    pub a: Vector2,
    pub b: Vector2,
}

impl Line2 {
    pub fn new(a: Vector2, b: Vector2) -> Self {
        Self { a, b }
    }

    pub fn from_base_dir(base: Vector2, dir: Vector2) -> Self {
        Self::new(base, base + dir)
    }

    pub fn dividing_mid(a: Vector2, b: &Vector2) -> Self {
        let mid = 0.5 * (a + b);
        let perp_dir = a - b;
        let dir = Vector2::new(-perp_dir.y, perp_dir.x);
        Self::from_base_dir(mid, dir)
    }

    pub fn intersection(&self, other: &Line2) -> Option<Vector2> {
        let det = |a, b, c, d| a * d - b * c;
        let denom = det(
            self.a.x - self.b.x,
            self.a.y - self.b.y,
            other.a.x - other.b.x,
            other.a.y - other.b.y,
        );

        if denom.abs() < std::f32::EPSILON {
            return None;
        }

        let a = det(self.a.x, self.a.y, self.b.x, self.b.y);
        let c = det(other.a.x, other.a.y, other.b.x, other.b.y);

        let x = det(a, self.a.x - self.b.x, c, other.a.x - other.b.x) / denom;
        let y = det(a, self.a.y - self.b.y, c, other.a.y - other.b.y) / denom;

        Some(Vector2::new(x, y))
    }
}

#[test]
fn line_intersection_test() {
    let test = |a: &Line2, b, e: Option<Vector2>, msg| {
        let i = a.intersection(b);
        match (i, e) {
            (None, None) => {}
            (Some(p), Some(q)) => {
                let dist = (p - q).magnitude();
                assert!(dist < 0.0001, "{}", msg);
            }
            _ => panic!("Unexpected case in {}", msg),
        }
    };

    let origin = Vector2::new(0.0, 0.0);
    let x_dir = Vector2::new(1.0, 0.0);
    let y_dir = Vector2::new(0.0, 1.0);

    let x_axis = Line2::from_base_dir(origin, x_dir);
    let y_axis = Line2::from_base_dir(origin, y_dir);
    let x_axis_para = Line2::from_base_dir(y_dir, x_dir);

    test(
        &x_axis,
        &y_axis,
        Some(origin),
        "X and Y axis intersect in origin",
    );

    test(&x_axis, &x_axis_para, None, "Parallel lines don't meet");

    let slanted = Line2::new(x_dir, Vector2::new(0.0, 15.0));

    test(
        &y_axis,
        &slanted,
        Some(Vector2::new(0.0, 15.0)),
        "Diagonal meets on y axis",
    );
}
