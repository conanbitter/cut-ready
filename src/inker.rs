use tiny_skia::{
    BlendMode, Color, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, StrokeDash, Transform,
};

use crate::{blueprint::Blueprint, camera::Camera, config::CONFIG, errors::AppError, geometry::EPSILON, gui::Gui};

pub struct Inker<'a> {
    pixmap: Pixmap,
    blueprint_pixmap: Pixmap,
    stroke: Stroke,
    frame_stroke: Stroke,
    pub camera: Camera,
    pub current_blueprint: Blueprint<'a>,
    error_paint: Paint<'a>,
    pub redraw_needed: bool,
    frame: Option<Path>,
    frame_paint: Paint<'a>,
}

impl<'a> Inker<'a> {
    pub fn new() -> anyhow::Result<Inker<'a>> {
        let pixmap = Pixmap::new(256, 256).ok_or(AppError::InkerNew)?;
        let blueprint_pixmap = Pixmap::new(256, 256).ok_or(AppError::InkerNew)?;

        let stroke = Stroke {
            width: 5.0,
            line_join: LineJoin::Round,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let error_paint = Paint {
            anti_alias: CONFIG.anti_aliasing,
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 0, 0, 255)),
            ..Paint::default()
        };

        let frame_paint = Paint {
            anti_alias: false,
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(255, 255, 255, 255)),
            blend_mode: BlendMode::Difference,
            ..Paint::default()
        };

        let frame_stroke = Stroke {
            width: 1.95,
            line_join: LineJoin::Miter,
            line_cap: LineCap::Butt,
            dash: StrokeDash::new(vec![10.0, 3.0], 0.0),
            ..Stroke::default()
        };

        Ok(Inker {
            pixmap,
            blueprint_pixmap,
            stroke,
            camera: Camera::default(),
            current_blueprint: Blueprint::default(),
            error_paint,
            redraw_needed: true,
            frame: None,
            frame_paint,
            frame_stroke,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(new_pixmap) = Pixmap::new(width, height)
            && let Some(new_blueprint_pixmap) = Pixmap::new(width, height)
        {
            self.pixmap = new_pixmap;
            self.blueprint_pixmap = new_blueprint_pixmap;
            self.camera.resize(width as f32, height as f32);
            self.redraw_needed = true;
        }
    }

    pub fn draw(&mut self, gui: &mut Gui, show_errors: bool, show_frame: bool) -> &[u8] {
        let force = self.redraw_needed;
        if self.redraw_needed {
            self.blueprint_pixmap.fill(Color::WHITE);

            for layer in &self.current_blueprint.layers {
                self.camera.apply_to_stroke(layer.line_width, &mut self.stroke);
                self.blueprint_pixmap
                    .stroke_path(&layer.drawing, &layer.color, &self.stroke, self.camera.matrix, None);
            }

            if show_errors {
                self.current_blueprint.errors.update(self.camera.matrix.get_scale().0);
                if let Some(error_path) = self.current_blueprint.errors.path.as_ref() {
                    self.camera.apply_to_stroke(CONFIG.line_widths.error, &mut self.stroke);
                    self.blueprint_pixmap.stroke_path(
                        error_path,
                        &self.error_paint,
                        &self.stroke,
                        self.camera.matrix,
                        None,
                    );
                }
            }

            self.pixmap.data_mut().copy_from_slice(self.blueprint_pixmap.data());

            self.redraw_needed = false;
        }

        if show_frame && let Some(frame_path) = self.frame.as_ref() {
            self.pixmap.stroke_path(
                frame_path,
                &self.frame_paint,
                &self.frame_stroke,
                Transform::identity(),
                None,
            );
        }

        gui.draw(&mut self.pixmap, force);

        self.pixmap.data()
    }

    pub fn show_all(&mut self) {
        self.camera.show_all(&self.current_blueprint.bounds);
    }

    pub fn update_frame(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let mut pb = if let Some(path) = self.frame.take() {
            path.clear()
        } else {
            PathBuilder::new()
        };

        let (left, right) = if (x1 - x2).abs() < EPSILON {
            (x1, x1 + EPSILON)
        } else if x1 < x2 {
            (x1, x2)
        } else {
            (x2, x1)
        };
        let (top, bottom) = if (y1 - y2).abs() < EPSILON {
            (y1, y1 + EPSILON)
        } else if y1 < y2 {
            (y1, y2)
        } else {
            (y2, y1)
        };
        //println!("Update l={}  t={}  r={}  b={}", left, top, right, bottom);

        let rect = Rect::from_ltrb(left, top, right, bottom).unwrap(); // checked before

        pb.push_rect(rect);
        self.frame = pb.finish();
    }

    pub fn zoom_frame(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if (x1 - x2).abs() < EPSILON || (y1 - y2).abs() < EPSILON {
            return;
        }

        let inv_camera = self.camera.matrix.invert().unwrap();
        let mut p1 = tiny_skia::Point { x: x1, y: y1 };
        let mut p2 = tiny_skia::Point { x: x2, y: y2 };
        inv_camera.map_point(&mut p1);
        inv_camera.map_point(&mut p2);
        let tiny_skia::Point { x: x1, y: y1 } = p1;
        let tiny_skia::Point { x: x2, y: y2 } = p2;

        let (left, right) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (top, bottom) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        //println!("Update l={}  t={}  r={}  b={}", left, top, right, bottom);

        let rect = Rect::from_ltrb(left, top, right, bottom).unwrap(); // checked before
        self.camera.show_all(&rect);
    }
}
