use dxf::entities::Spline;
use enterpolation::{Curve, Signal, bspline::BSpline};
use tiny_skia::PathBuilder;

use crate::geometry::{EPSILON, Point2D};

const MAX_DEPTH: usize = 100;
const LINEAR_TOLERANCE: f64 = 0.01;
const LINEAR_TOLERANCE2: f64 = LINEAR_TOLERANCE * LINEAR_TOLERANCE;

fn process_segment<C>(curve: &C, start: f64, end: f64, depth: usize, points: &mut Vec<f64>)
where
    C: Signal<f64, Output = Point2D>,
{
    if depth > MAX_DEPTH {
        return;
    }

    const CHECKS: [f64; 7] = [
        1.0 / 8.0,
        2.0 / 8.0,
        3.0 / 8.0,
        4.0 / 8.0,
        5.0 / 8.0,
        6.0 / 8.0,
        7.0 / 8.0,
    ];

    let mut max_error: f64 = 0.0;
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
}

fn dedup_and_sort(vec: &mut Vec<f64>) {
    vec.sort_by(|a, b| a.total_cmp(b));
    vec.dedup_by(|a, b| (*a - *b).abs() < EPSILON);
}

fn linear_segmentation<C>(curve: &C, builder: &mut PathBuilder, knots: &Vec<f64>)
where
    C: Signal<f64, Output = Point2D> + Curve<f64>,
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

    points.sort_by(|a, b| a.total_cmp(b));

    let start_point = curve.eval(start);
    builder.move_to(start_point.x as f32, -start_point.y as f32);

    for p in points {
        let point = curve.eval(p);
        builder.line_to(point.x as f32, -point.y as f32);
    }
}

pub fn curve_path(dxf_spline: &Spline) -> anyhow::Result<tiny_skia::Path> {
    let mut pb = PathBuilder::new();

    if dxf_spline.flags & 4 > 0 {
        // NURBS
        let control_points: Vec<(Point2D, f64)> = dxf_spline
            .control_points
            .iter()
            .zip(dxf_spline.weight_values.iter())
            .map(|(p, w)| (Point2D::new(p.x, p.y), *w))
            .collect();

        let curve = BSpline::builder()
            .legacy()
            .elements_with_weights(control_points)
            .knots(&dxf_spline.knot_values)
            .dynamic()
            .build()?;

        linear_segmentation(&curve, &mut pb, &dxf_spline.knot_values);
    } else {
        // B-Slines
        let control_points: Vec<Point2D> = dxf_spline
            .control_points
            .iter()
            .map(|p| Point2D::new(p.x, p.y))
            .collect();

        let curve = BSpline::builder()
            .legacy()
            .elements(control_points)
            .knots(&dxf_spline.knot_values)
            .dynamic()
            .build()?;

        linear_segmentation(&curve, &mut pb, &dxf_spline.knot_values);
    }
    Ok(pb.finish().unwrap())
}
