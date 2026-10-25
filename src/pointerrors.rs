use tiny_skia::PathBuilder;

use crate::geometry::Point2D;

const CONNECT_TRESHOLD: f32 = 0.0001;
const CONNECT_TRESHOLD2: f32 = CONNECT_TRESHOLD * CONNECT_TRESHOLD;

const MARKER_RADIUS: f32 = 15.0;
pub const ERROR_LINE_WIDTH: f32 = 2.95;

pub struct PointErrors {
    points: Vec<Point2D>,
    pub path: Option<tiny_skia::Path>,
    old_scale: f32,
    current_error: usize,
}

impl PointErrors {
    pub fn new(points: Vec<Point2D>) -> PointErrors {
        let path = if points.is_empty() {
            None
        } else {
            let mut pb = PathBuilder::new();
            PointErrors::draw(&points, &mut pb, 1.0);
            pb.finish()
        };
        PointErrors {
            current_error: points.len(),
            points,
            old_scale: 1.0,
            path,
        }
    }

    fn draw(points: &[Point2D], pb: &mut PathBuilder, scale: f32) {
        let radius = MARKER_RADIUS / scale;

        for point in points {
            pb.push_circle(point.x, point.y, radius);
            pb.move_to(point.x - radius, point.y - radius);
            pb.line_to(point.x + radius, point.y + radius);
            pb.move_to(point.x + radius, point.y - radius);
            pb.line_to(point.x - radius, point.y + radius);
        }
    }

    pub fn update(&mut self, new_scale: f32) {
        if new_scale != self.old_scale
            && let Some(path) = self.path.take()
        {
            let mut pb = path.clear();
            PointErrors::draw(&self.points, &mut pb, new_scale);
            self.path = Some(pb.finish().unwrap());
            self.old_scale = new_scale;
        }
    }

    pub fn next_error(&mut self) -> Point2D {
        if self.current_error >= self.points.len() - 1 {
            self.current_error = 0;
        } else {
            self.current_error += 1;
        }
        self.points[self.current_error]
    }

    pub fn prev_error(&mut self) -> Point2D {
        if self.current_error == 0 {
            self.current_error = self.points.len() - 1;
        } else {
            self.current_error -= 1;
        }
        self.points[self.current_error]
    }
}

impl Default for PointErrors {
    fn default() -> Self {
        Self {
            points: Default::default(),
            path: None,
            old_scale: 1.0,
            current_error: 0,
        }
    }
}

pub fn collect_errors(open_ends: &[Point2D]) -> Vec<Point2D> {
    let mut removed = vec![false; open_ends.len()];

    for (i, p1) in open_ends.iter().enumerate() {
        if removed[i] {
            continue;
        }

        for (j, p2) in open_ends.iter().enumerate().skip(i + 1) {
            if removed[j] {
                continue;
            }

            if p1.dist2(*p2) < CONNECT_TRESHOLD2 {
                removed[i] = true;
                removed[j] = true;
                break;
            }
        }
    }

    open_ends
        .iter()
        .zip(removed.iter())
        .filter_map(|(&point, &is_removed)| if is_removed { None } else { Some(point) })
        .collect()
}
