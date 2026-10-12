use tiny_skia::{Color, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

pub struct Inker {
    pixmap: Pixmap,
    path: Path,
    stroke: Stroke,
}

impl Inker {
    pub fn new() -> Inker {
        let pixmap = Pixmap::new(256, 256).unwrap();

        let mut pb = PathBuilder::new();
        pb.move_to(128.0, 32.0);
        pb.line_to(224.0, 224.0);
        pb.line_to(32.0, 224.0);
        pb.close();
        let path = pb.finish().unwrap();

        let stroke = Stroke {
            width: 5.0,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };

        Inker { pixmap, path, stroke }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width, height).unwrap();
    }

    pub fn draw(&mut self) -> &[u8] {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 255, 255); //(50, 127, 150, 255);
        paint.anti_alias = true;
        self.pixmap.fill(Color::WHITE);
        self.pixmap
            .stroke_path(&self.path, &paint, &self.stroke, Transform::identity(), None);
        //self.pixmap.save_png("test.png").unwrap();
        self.pixmap.data()
    }
}
