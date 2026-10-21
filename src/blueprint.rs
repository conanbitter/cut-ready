use dxf::{Drawing, entities::EntityType};
use std::path::Path;
use tiny_skia::PathBuilder;

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
                //
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
