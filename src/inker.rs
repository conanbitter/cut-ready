use tiny_skia::{Color, LineCap, LineJoin, Pixmap, Stroke};

use crate::{blueprint::Blueprint, camera::Camera};

pub struct Inker<'a> {
    pixmap: Pixmap,
    stroke: Stroke,
    pub camera: Camera,
    pub current_blueprint: Blueprint<'a>,
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
            camera: Camera::default(),
            current_blueprint: bp,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width, height).unwrap();
        self.camera.resize(width, height);
    }

    pub fn draw(&mut self) -> &[u8] {
        self.pixmap.fill(Color::WHITE);
        for layer in &self.current_blueprint.layers {
            self.camera.apply_to_stroke(layer.line_width, &mut self.stroke);
            self.pixmap
                .stroke_path(&layer.drawing, &layer.color, &self.stroke, self.camera.matrix, None);
        }

        self.pixmap.data()
    }

    pub fn show_all(&mut self) {
        self.camera.show_all(&self.current_blueprint.bounds);
    }
}
