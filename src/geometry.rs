pub const EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Default)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Point2D {
        Point2D { x, y }
    }

    pub fn dist2(self, other: Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn transform(self, transform: tiny_skia::Transform) -> Point2D {
        if transform.is_identity() {
            self
        } else if transform.is_translate() {
            Point2D {
                x: self.x + transform.tx,
                y: self.y + transform.ty,
            }
        } else if transform.is_scale_translate() {
            Point2D {
                x: self.x * transform.sx + transform.tx,
                y: self.y * transform.sy + transform.ty,
            }
        } else {
            Point2D {
                x: self.x * transform.sx + self.y * transform.kx + transform.tx,
                y: self.x * transform.ky + self.y * transform.sy + transform.ty,
            }
        }
    }
}

impl std::ops::Add for Point2D {
    type Output = Point2D;

    fn add(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Point2D {
    type Output = Point2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Point2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Point2D {
    type Output = Point2D;

    fn mul(self, rhs: f32) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f32> for Point2D {
    type Output = Point2D;

    fn div(self, rhs: f32) -> Self::Output {
        Point2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::fmt::Debug for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point2D")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
