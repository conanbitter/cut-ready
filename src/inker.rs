use tiny_skia::{Color, LineCap, LineJoin, Paint, Pixmap, Stroke};

use crate::{blueprint::Blueprint, camera::Camera, gui::Gui, pointerrors::ERROR_LINE_WIDTH};

pub struct Inker<'a> {
    pixmap: Pixmap,
    stroke: Stroke,
    pub camera: Camera,
    pub current_blueprint: Blueprint<'a>,
    error_paint: Paint<'a>,
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

        let error_paint = Paint {
            anti_alias: true,
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 0, 0, 255)),
            ..Paint::default()
        };

        Inker {
            pixmap,
            stroke,
            camera: Camera::default(),
            current_blueprint: bp,
            error_paint,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixmap = Pixmap::new(width, height).unwrap();
        self.camera.resize(width, height);
    }

    pub fn draw(&mut self, gui: &Gui) -> &[u8] {
        self.pixmap.fill(Color::WHITE);

        for layer in &self.current_blueprint.layers {
            self.camera.apply_to_stroke(layer.line_width, &mut self.stroke);
            self.pixmap
                .stroke_path(&layer.drawing, &layer.color, &self.stroke, self.camera.matrix, None);
        }

        self.current_blueprint.errors.update(self.camera.matrix.get_scale().0);
        if let Some(error_path) = self.current_blueprint.errors.path.as_ref() {
            self.camera.apply_to_stroke(ERROR_LINE_WIDTH, &mut self.stroke);
            self.pixmap
                .stroke_path(error_path, &self.error_paint, &self.stroke, self.camera.matrix, None);
        }

        gui.draw(&mut self.pixmap);

        self.pixmap.data()
    }

    pub fn show_all(&mut self) {
        self.camera.show_all(&self.current_blueprint.bounds);
    }
}
