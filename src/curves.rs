use dxf::entities::Spline;
use enterpolation::{Curve, Signal, bspline::BSpline};
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

    fn dist2(self, other: Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
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

impl std::ops::Sub for Point2D {
    type Output = Point2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
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

const MAX_DEPTH: usize = 100;
const LINEAR_TOLERANCE: f32 = 0.01;
const LINEAR_TOLERANCE2: f32 = LINEAR_TOLERANCE * LINEAR_TOLERANCE;

fn process_segment<C>(curve: &C, start: f32, end: f32, depth: usize, points: &mut Vec<f32>)
where
    C: Signal<f32, Output = Point2D>,
{
    if depth > MAX_DEPTH {
        return;
    }

    const CHECKS: [f32; 7] = [0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875];

    let mut max_error: f32 = 0.0;
    let start_point = curve.eval(start);
    let end_point = curve.eval(end);

    for u in CHECKS {
        let t = start + (end - start) * u;

        let curve_point = curve.eval(t);
        let segment_point = start_point + (end_point - start_point) * u;

        max_error = max_error.max(curve_point.dist2(segment_point));
    }

    if max_error > LINEAR_TOLERANCE2 {
        let middle_point = (start + end) / 2.0;
        points.push(middle_point);
        process_segment(curve, start, middle_point, depth + 1, points);
        process_segment(curve, middle_point, end, depth + 1, points);
    }

    /*let middle_point = (start + end) / 2.0;
    let curve_middle = curve.eval(middle_point);
    let segment_middle = (curve.eval(start) + curve.eval(end)) / 2.0;
    if curve_middle.dist2(segment_middle) > LINEAR_TOLERANCE2 {
        points.push(middle_point);
        process_segment(curve, start, middle_point, depth + 1, points);
        process_segment(curve, middle_point, end, depth + 1, points);
    }*/
}

fn dedup_and_sort(vec: &mut Vec<f32>) {
    const EPSILON: f32 = 0.00001;

    vec.sort_by(|a, b| a.total_cmp(b));

    vec.dedup_by(|a, b| (*a - *b).abs() < EPSILON);
}

fn linear_segmentation<C>(curve: &C, builder: &mut PathBuilder, knots: &Vec<f32>)
where
    C: Signal<f32, Output = Point2D> + Curve<f32>,
{
    let [start, end] = curve.domain();

    let mut segments = vec![start, end];
    segments.extend(knots);
    dedup_and_sort(&mut segments);

    let mut points = vec![];

    for s in segments.windows(2) {
        points.push(s[1]);
        process_segment(curve, s[0], s[1], 0, &mut points);
    }

    //points.push(end);
    //process_segment(curve, start, end, 0, &mut points);
    points.sort_by(|a, b| a.total_cmp(b));
    //println!("{:?}", points);
    //println!("Segments {}", points.len() - 1);

    let start_point = curve.eval(start);
    builder.move_to(start_point.x, start_point.y);

    for p in points {
        let point = curve.eval(p);
        builder.line_to(point.x, point.y);
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
            .knots(&knots)
            .dynamic()
            .build()?;
        linear_segmentation(&curve, &mut pb, &knots);
        /*for (i, p) in curve.take(100).enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }*/
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
            .knots(&knots)
            .dynamic()
            .build()?;
        linear_segmentation(&curve, &mut pb, &knots);
        /*for (i, p) in curve.take(100).enumerate() {
            if i == 0 {
                pb.move_to(p.x, p.y);
            } else {
                pb.line_to(p.x, p.y);
            }
        }*/
    }
    Ok(pb.finish().unwrap())
}
