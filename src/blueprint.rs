use dxf::{Drawing, entities::EntityType};
use std::path::Path;
use tiny_skia::PathBuilder;

const KAPPA: f64 = 0.5522847498307934;

fn draw_arc(cx: f64, cy: f64, r: f64, pb: &mut PathBuilder) {
    let kr = KAPPA * r;

    pb.move_to((cx + r) as f32, cy as f32);
    pb.cubic_to(
        (cx + r) as f32,
        (cy + kr) as f32,
        (cx + kr) as f32,
        (cy + r) as f32,
        cx as f32,
        (cy + r) as f32,
    );
}

pub fn load_dxf(filename: impl AsRef<Path>) -> anyhow::Result<tiny_skia::Path> {
    let drawing = Drawing::load_file(filename).unwrap();

    let mut pb = PathBuilder::new();

    for e in drawing.entities() {
        match e.specific {
            EntityType::Circle(ref circle) => {
                pb.push_circle(circle.center.x as f32, circle.center.y as f32, circle.radius as f32);
            }
            EntityType::Line(ref line) => {
                pb.move_to(line.p1.x as f32, line.p1.y as f32);
                pb.line_to(line.p2.x as f32, line.p2.y as f32);
            }
            EntityType::Arc(ref arc) => {
                draw_arc(arc.center.x, arc.center.y, arc.radius, &mut pb);
            }
            EntityType::Ellipse(ref arc) => {
                //
            }
            EntityType::LwPolyline(ref polyline) => {
                //
            }
            EntityType::Spline(ref spline) => {
                //
            }
            _ => {
                println!("======\n{:?}", e.specific);
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
