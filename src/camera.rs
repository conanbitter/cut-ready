use tiny_skia::Transform;

use crate::geometry::Point2D;

pub struct Camera {
    offsetx: f32,
    offsety: f32,
    scale_factor: f32,
    pub matrix: Transform,
    viewport_width: f32,
    viewport_height: f32,
    viewport_offset: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offsetx: 0.0,
            offsety: 0.0,
            scale_factor: 1.0,
            matrix: Transform::identity(),
            viewport_width: 0.0,
            viewport_height: 0.0,
            viewport_offset: 0.0,
        }
    }
}

impl Camera {
    fn update_matrix(&mut self) {
        self.matrix =
            Transform::from_scale(self.scale_factor, self.scale_factor).post_translate(self.offsetx, self.offsety);
    }

    pub fn pan(&mut self, x: f32, y: f32) {
        self.offsetx += x;
        self.offsety += y;
        self.update_matrix();
    }

    pub fn adjust_offset(&mut self, x: f32, y: f32, k: f32) {
        self.offsetx = x * (1.0 - k) + self.offsetx * k;
        self.offsety = y * (1.0 - k) + self.offsety * k;
    }

    pub fn scale(&mut self, ds: f32, x: f32, y: f32) {
        self.scale_factor *= ds;
        self.adjust_offset(x, y, ds);
        self.update_matrix();
    }

    pub fn apply_to_stroke(&self, width: f32, stroke: &mut tiny_skia::Stroke) {
        stroke.width = width / self.scale_factor;
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, offset: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.viewport_offset = offset;
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let old_width = self.viewport_width;
        let old_height = self.viewport_height;

        self.offsetx += (width - old_width) / 2.0;
        self.offsety += (height - old_height) / 2.0;

        let scale_adjust = f32::min(width / old_width, height / old_height);
        self.scale_factor *= scale_adjust;
        self.adjust_offset(width / 2.0, height / 2.0 + self.viewport_offset, scale_adjust);

        self.viewport_width = width;
        self.viewport_height = height;
        self.update_matrix();
    }

    pub fn show_all(&mut self, bounds: &tiny_skia::Rect) {
        const VIEW_BORDER: f32 = 20.0;

        self.scale_factor = f32::min(
            (self.viewport_width - VIEW_BORDER * 2.0) / bounds.width(),
            (self.viewport_height - self.viewport_offset - VIEW_BORDER * 2.0) / bounds.height(),
        );

        self.offsetx = self.viewport_width / 2.0 - (bounds.left() + bounds.width() / 2.0) * self.scale_factor;
        self.offsety = (self.viewport_height + self.viewport_offset) / 2.0
            - (bounds.top() + bounds.height() / 2.0) * self.scale_factor;

        self.update_matrix();
    }

    pub fn move_to(&mut self, point: Point2D) {
        self.offsetx = self.viewport_width / 2.0 - point.x * self.scale_factor;
        self.offsety = self.viewport_height / 2.0 - point.y * self.scale_factor;
        self.update_matrix();
    }
}
