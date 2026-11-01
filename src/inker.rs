use tiny_skia::{
    BlendMode, Color, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, StrokeDash, Transform,
};

use crate::{blueprint::Blueprint, camera::Camera, config::CONFIG, errors::AppError, geometry::EPSILON, gui::Gui};

pub struct Inker<'a> {
    main_pixmap: Pixmap,
    stroke: Stroke,
    camera: Camera,

    show_errors: bool,
    error_paint: Paint<'a>,

    pub current_blueprint: Blueprint<'a>,
    blueprint_pixmap: Pixmap,

    frame: Option<Path>,
    frame_paint: Paint<'a>,
    frame_stroke: Stroke,
    frame_show: bool,

    window_dirty: bool,
    blueprint_dirty: bool,
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
            main_pixmap: pixmap,
            stroke,
            camera: Camera::default(),

            show_errors: false,
            error_paint,

            current_blueprint: Blueprint::default(),
            blueprint_pixmap,

            frame: None,
            frame_paint,
            frame_stroke,
            frame_show: false,

            blueprint_dirty: true,
            window_dirty: false,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(new_pixmap) = Pixmap::new(width, height)
            && let Some(new_blueprint_pixmap) = Pixmap::new(width, height)
        {
            self.main_pixmap = new_pixmap;
            self.blueprint_pixmap = new_blueprint_pixmap;
            self.camera.resize(width as f32, height as f32);
            self.blueprint_dirty = true;
        }
    }

    pub fn draw(&mut self, gui: &mut Gui) -> &[u8] {
        if self.blueprint_dirty {
            self.window_dirty = true;

            self.blueprint_pixmap.fill(Color::WHITE);

            for layer in &self.current_blueprint.layers {
                self.camera.apply_to_stroke(layer.line_width, &mut self.stroke);
                self.blueprint_pixmap
                    .stroke_path(&layer.drawing, &layer.color, &self.stroke, self.camera.matrix, None);
            }

            if self.show_errors {
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

            self.blueprint_dirty = false;
        }

        let force_gui = self.window_dirty;

        if self.window_dirty {
            self.main_pixmap
                .data_mut()
                .copy_from_slice(self.blueprint_pixmap.data());

            if self.frame_show
                && let Some(frame_path) = self.frame.as_ref()
            {
                self.main_pixmap.stroke_path(
                    frame_path,
                    &self.frame_paint,
                    &self.frame_stroke,
                    Transform::identity(),
                    None,
                );
            }

            self.window_dirty = false;
        }

        gui.draw(&mut self.main_pixmap, force_gui);

        self.main_pixmap.data()
    }

    pub fn show_all(&mut self) {
        self.camera.show_all(&self.current_blueprint.bounds);
        self.blueprint_dirty = true;
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
        self.frame_show = true;
        self.window_dirty = true;
    }

    pub fn zoom_frame(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if (x1 - x2).abs() < EPSILON || (y1 - y2).abs() < EPSILON {
            self.frame_show = false;
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
        self.frame_show = false;
        self.blueprint_dirty = true;
    }

    pub fn errors_visibile(&mut self, visible: bool) {
        self.show_errors = visible;
        self.blueprint_dirty = true;
    }

    pub fn pan(&mut self, x: f32, y: f32) {
        self.camera.pan(x, y);
        self.blueprint_dirty = true;
    }

    pub fn zoom(&mut self, ds: f32, x: f32, y: f32) {
        self.camera.scale(ds, x, y);
        self.blueprint_dirty = true;
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, offset: f32) {
        self.camera.set_viewport(width, height, offset);
    }

    pub fn invalidate(&mut self) {
        self.window_dirty = true;
    }

    pub fn set_blueprint(&mut self, new_blueprint: Blueprint<'a>) {
        self.current_blueprint = new_blueprint;
        self.show_all();
    }

    pub fn next_error(&mut self) {
        if self.show_errors
            && let Some(point) = self.current_blueprint.errors.next_error()
        {
            self.camera.move_to(point);
            self.blueprint_dirty = true;
        }
    }

    pub fn prev_error(&mut self) {
        if self.show_errors
            && let Some(point) = self.current_blueprint.errors.prev_error()
        {
            self.camera.move_to(point);
            self.blueprint_dirty = true;
        }
    }
}
