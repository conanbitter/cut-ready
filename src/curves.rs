use dxf::entities::Spline;
use enterpolation::{Curve, bspline::BSpline};
use tiny_skia::PathBuilder;

#[derive(Clone, Copy, Default)]
struct Point2D {
    x: f32,
    y: f32,
}

impl Point2D {
    fn new(x: f64, y: f64) -> Point2D {
        Point2D {
            x: x as f32,
            y: y as f32,
        }
    }
}

impl std::ops::Add for Point2D {
    type Output = Point2D;

    fn add(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Point2D {
    type Output = Point2D;

    fn mul(self, rhs: f32) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f32> for Point2D {
    type Output = Point2D;

    fn div(self, rhs: f32) -> Self::Output {
        Point2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::fmt::Debug for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point2D")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

pub fn curve_path(dxf_spline: &Spline) -> anyhow::Result<tiny_skia::Path> {
    let knots: Vec<f32> = dxf_spline.knot_values.iter().map(|k| *k as f32).collect();

    let mut pb = PathBuilder::new();

    if dxf_spline.flags & 4 > 0 {
        // NURBS
        let control_points: Vec<(Point2D, f32)> = dxf_spline
            .control_points
            .iter()
            .zip(dxf_spline.weight_values.iter())
            .map(|(p, w)| (Point2D::new(p.x, -p.y), *w as f32))
            .collect();
        let curve = BSpline::builder()
            .legacy()
            .elements_with_weights(control_points)
            .knots(knots)
            .dynamic()
            .build()?;
        for (i, p) in curve.take(100).enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }
    } else {
        // B-Slines
        let control_points: Vec<Point2D> = dxf_spline
            .control_points
            .iter()
            .map(|p| Point2D::new(p.x, -p.y))
            .collect();

        let curve = BSpline::builder()
            .legacy()
            .elements(control_points)
            .knots(knots)
            .dynamic()
            .build()?;

        for (i, p) in curve.take(100).enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }
    }
    Ok(pb.finish().unwrap())
}
