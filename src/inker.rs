use tiny_skia::{Color, LineCap, LineJoin, Pixmap, Stroke, Transform};

use crate::blueprint::Blueprint;

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

        self.stroke.width = 0.95 / self.scale;

        self.pixmap.fill(Color::WHITE);
        for layer in &self.current_blueprint.layers {
            self.pixmap
                .stroke_path(&layer.drawing, &layer.color, &self.stroke, transform, None);
        }

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
