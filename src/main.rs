use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

fn main() {
    let mut pixmap = Pixmap::new(256, 256).unwrap();
    pixmap.fill(Color::WHITE);

    let mut pb = PathBuilder::new();
    pb.move_to(128.0, 32.0);
    pb.line_to(224.0, 224.0);
    pb.line_to(32.0, 224.0);
    pb.close();
    let path = pb.finish().unwrap();

    let mut paint = Paint::default();
    paint.set_color_rgba8(50, 127, 150, 200);
    paint.anti_alias = true;

    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

    pixmap.save_png("triangle.png").unwrap();
}
