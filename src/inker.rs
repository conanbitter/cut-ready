use std::sync::LazyLock;

use tiny_skia::{Color, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

static COLOR_MAIN: LazyLock<Paint> = LazyLock::new(|| paint_from_color(0, 100, 255));

fn paint_from_color(r: u8, g: u8, b: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint.anti_alias = true;
    paint
}

pub struct Inker {
    pixmap: Pixmap,
    path: Path,
    stroke: Stroke,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
}

impl Inker {
    pub fn new(path: Path) -> Inker {
        let pixmap = Pixmap::new(256, 256).unwrap();

        let stroke = Stroke {
            width: 5.0,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };

        Inker {
            pixmap,
            path,
            stroke,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width, height).unwrap();
    }

    pub fn draw(&mut self) -> &[u8] {
        let transform = Transform::from_translate(self.offset_x, self.offset_y).post_scale(self.scale, self.scale);

        self.stroke.width = 5.0 / self.scale;

        self.pixmap.fill(Color::WHITE);
        self.pixmap
            .stroke_path(&self.path, &COLOR_MAIN, &self.stroke, transform, None);
        //self.pixmap.save_png("test.png").unwrap();
        self.pixmap.data()
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx / self.scale;
        self.offset_y += dy / self.scale;
    }

    pub fn scale(&mut self, dscale: f32) {
        self.scale *= dscale;
    }
}
