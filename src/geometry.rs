pub const EPSILON: f64 = 0.0001;

#[derive(Clone, Copy, Default)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Point2D {
        Point2D { x, y }
    }

    pub fn dist2(self, other: Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
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

impl std::ops::Mul<f64> for Point2D {
    type Output = Point2D;

    fn mul(self, rhs: f64) -> Self::Output {
        Point2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f64> for Point2D {
    type Output = Point2D;

    fn div(self, rhs: f64) -> Self::Output {
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
