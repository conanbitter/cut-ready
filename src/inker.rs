use std::sync::LazyLock;

use tiny_skia::{Color, LineCap, LineJoin, Paint, Path, Pixmap, Stroke, Transform};

use crate::blueprint::Blueprint;

static COLOR_MAIN: LazyLock<Paint> = LazyLock::new(|| paint_from_color(0, 100, 255));

fn paint_from_color(r: u8, g: u8, b: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint.anti_alias = true;
    paint
}

pub struct Inker<'a> {
    pixmap: Pixmap,
    stroke: Stroke,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    current_blueprint: Blueprint<'a>,
}

impl<'a> Inker<'a> {
    pub fn new(bp: Blueprint<'a>) -> Inker<'a> {
        let pixmap = Pixmap::new(256, 256).unwrap();

        let stroke = Stroke {
            width: 5.0,
            line_join: LineJoin::Round,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        Inker {
            pixmap,
            stroke,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            current_blueprint: bp,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width, height).unwrap();
    }

    pub fn draw(&mut self) -> &[u8] {
        let transform = Transform::from_translate(self.offset_x, self.offset_y).post_scale(self.scale, self.scale);

        self.stroke.width = 2.0 / self.scale;

        self.pixmap.fill(Color::WHITE);
        for layer in &self.current_blueprint.layers {
            self.pixmap
                .stroke_path(&layer.drawing, &layer.color, &self.stroke, transform, None);
        }
        //self.pixmap
        //    .stroke_path(&self.path, &COLOR_MAIN, &self.stroke, transform, None);
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
