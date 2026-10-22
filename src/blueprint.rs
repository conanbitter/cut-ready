use dxf::{Drawing, entities::EntityType};
use std::{f64::consts::TAU, path::Path};
use tiny_skia::{PathBuilder, Transform};

use crate::curves::curve_path;

const MAX_SEGMENT_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
const EPSILON: f64 = 0.0001;

fn arc_path(cx: f32, cy: f32, r: f32, start_angle: f32, end_angle: f32) -> tiny_skia::Path {
    let sweep_angle = if start_angle <= end_angle {
        end_angle - start_angle
    } else {
        end_angle + std::f32::consts::TAU - start_angle
    };

    let mut pb = PathBuilder::new();

    let segments_count = (sweep_angle / MAX_SEGMENT_ANGLE).ceil() as usize;
    let segment_sweep = sweep_angle / segments_count as f32;

    let lr = (4.0 / 3.0) * (segment_sweep / 4.0).tan() * r;

    let mut cur_angle = start_angle;
    let start_x = cx + r * cur_angle.cos();
    let start_y = cy + r * cur_angle.sin();
    pb.move_to(start_x, -start_y);

    for _ in 0..segments_count {
        let next_angle = cur_angle + segment_sweep;

        let (sin_curr, cos_curr) = cur_angle.sin_cos();
        let (sin_next, cos_next) = next_angle.sin_cos();

        let x0 = cx + r * cos_curr;
        let y0 = cy + r * sin_curr;

        let x = cx + r * cos_next;
        let y = cy + r * sin_next;

        let x1 = x0 - lr * sin_curr;
        let y1 = y0 + lr * cos_curr;

        let x2 = x + lr * sin_next;
        let y2 = y - lr * cos_next;

        pb.cubic_to(x1, -y1, x2, -y2, x, -y);
        //pb.line_to(x, y);

        cur_angle = next_angle;
    }

    pb.finish().unwrap()
}

pub fn load_dxf(filename: impl AsRef<Path>) -> anyhow::Result<tiny_skia::Path> {
    let drawing = Drawing::load_file(filename).unwrap();

    let mut pb = PathBuilder::new();

    for e in drawing.entities() {
        match e.specific {
            EntityType::Circle(ref circle) => {
                pb.push_circle(circle.center.x as f32, -circle.center.y as f32, circle.radius as f32);
            }
            EntityType::Line(ref line) => {
                pb.move_to(line.p1.x as f32, -line.p1.y as f32);
                pb.line_to(line.p2.x as f32, -line.p2.y as f32);
            }
            EntityType::Arc(ref arc) => {
                pb.push_path(&arc_path(
                    arc.center.x as f32,
                    arc.center.y as f32,
                    arc.radius as f32,
                    arc.start_angle.to_radians() as f32,
                    arc.end_angle.to_radians() as f32,
                ));
            }
            EntityType::Ellipse(ref ellipse) => {
                let radius = (ellipse.major_axis.x * ellipse.major_axis.x + ellipse.major_axis.y * ellipse.major_axis.y)
                    .sqrt() as f32;
                let angle = {
                    let res = (ellipse.major_axis.x as f32 / radius).acos();
                    if ellipse.major_axis.y > 0.0 { -res } else { res }
                };

                let path = if ellipse.start_parameter.abs() < EPSILON && (ellipse.end_parameter - TAU).abs() < EPSILON {
                    PathBuilder::from_circle(0.0, 0.0, radius).unwrap()
                } else {
                    arc_path(
                        0.0,
                        0.0,
                        radius,
                        ellipse.start_parameter as f32,
                        ellipse.end_parameter as f32,
                    )
                };

                let transform = Transform::from_scale(1.0, ellipse.minor_axis_ratio as f32)
                    .post_rotate(angle.to_degrees())
                    .post_translate(ellipse.center.x as f32, -ellipse.center.y as f32);

                pb.push_path(&path.transform(transform).unwrap());
            }
            EntityType::LwPolyline(ref polyline) => {
                if !polyline.vertices.is_empty() {
                    let mut line = PathBuilder::new();
                    line.move_to(polyline.vertices[0].x as f32, -polyline.vertices[0].y as f32);
                    for point in &polyline.vertices {
                        line.line_to(point.x as f32, -point.y as f32);
                    }
                    if polyline.flags > 0 {
                        line.close();
                    }
                    pb.push_path(&line.finish().unwrap());
                }
            }
            EntityType::Spline(ref spline) => {
                /*println!(
                    "{} {} degree:{:3}/{:3}  cps:{:3} fits:{:3} knots:{:3} weights:{:3} valid: {}<={}<={}",
                    if spline.flags & 4 > 0 { "NURBS   " } else { "B-Spline" },
                    if spline.flags & 1 > 0 { "closed" } else { "open  " },
                    spline.degree_of_curve,
                    spline.knot_values.len() - spline.control_points.len() - 1,
                    spline.control_points.len(),
                    spline.fit_points.len(),
                    spline.knot_values.len(),
                    spline.weight_values.len(),
                    spline.control_points.len(),
                    spline.knot_values.len(),
                    2 * (spline.control_points.len() - 1),
                );*/
                pb.push_path(&curve_path(spline).unwrap());
            }
            _ => {
                //println!("======\n{:?}", e.specific);
            }
        }
    }

    if pb.is_empty() {
        pb.push_circle(100.0, 100.0, 50.0);
        pb.move_to(128.0, 32.0);
        pb.line_to(224.0, 224.0);
        pb.line_to(32.0, 224.0);
        pb.close();
    }

    Ok(pb.finish().unwrap())
}
