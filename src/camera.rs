use tiny_skia::Transform;

pub struct Camera {
    offsetx: f32,
    offsety: f32,
    scale_factor: f32,
    pub matrix: Transform,
    viewport_width: u32,
    viewport_height: u32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            offsetx: 0.0,
            offsety: 0.0,
            scale_factor: 1.0,
            matrix: Transform::identity(),
            viewport_width: 0,
            viewport_height: 0,
        }
    }

    fn update_matrix(&mut self) {
        self.matrix =
            Transform::from_translate(self.offsetx, self.offsety).post_scale(self.scale_factor, self.scale_factor);
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

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.offsetx += (width - self.viewport_width) as f32 / 2.0;
        self.offsety += (height - self.viewport_height) as f32 / 2.0;
        self.viewport_width = width;
        self.viewport_height = height;
        self.update_matrix();
    }

    pub fn show_all(&mut self, bounds: &tiny_skia::Rect) {
        const VIEW_BORDER: u32 = 20;

        self.scale_factor = f32::min(
            (self.viewport_width - VIEW_BORDER * 2) as f32 / bounds.width(),
            (self.viewport_height - VIEW_BORDER * 2) as f32 / bounds.height(),
        );

        self.offsetx = self.viewport_width as f32 / 2.0 - (bounds.left() + bounds.width() / 2.0) * self.scale_factor;
        self.offsety = self.viewport_height as f32 / 2.0 - (bounds.top() + bounds.height() / 2.0) * self.scale_factor;

        self.update_matrix();
    }
}
