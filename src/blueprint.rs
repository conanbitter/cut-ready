use dxf::{
    Drawing,
    entities::{Entity, EntityType},
};
use std::{collections::HashMap, f64::consts::TAU, path::Path};
use tiny_skia::{PathBuilder, Transform};

use crate::{curves::curve_path, geometry::Point2D};

const MAX_SEGMENT_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
const EPSILON: f64 = 0.0001;

struct DrawingBlock {
    layers: HashMap<i32, tiny_skia::PathBuilder>,
    open_ends: Vec<Point2D>,
}

struct FinishedBlock {
    layers: Vec<(i32, tiny_skia::Path)>,
    open_ends: Vec<Point2D>,
}

struct BlueprintLayer<'a> {
    drawing: tiny_skia::Path,
    color: tiny_skia::Paint<'a>,
}
struct Blueprint<'a> {
    layers: Vec<BlueprintLayer<'a>>,
}

impl DrawingBlock {
    fn new() -> DrawingBlock {
        DrawingBlock {
            layers: HashMap::new(),
            open_ends: vec![],
        }
    }

    fn request_path(&mut self, color: i32) -> &mut PathBuilder {
        self.layers.entry(color).or_default();
        self.layers.get_mut(&color).unwrap()
    }

    fn finish(self) -> anyhow::Result<FinishedBlock> {
        let mut layers = vec![];
        for (color, builder) in self.layers {
            layers.push((color, builder.finish().ok_or(anyhow::anyhow!("Path is empty"))?));
        }
        Ok(FinishedBlock {
            layers,
            open_ends: self.open_ends,
        })
    }
}

fn is_primary_color(color: i32) -> bool {
    color != 0
}

fn arc_draw(cx: f32, cy: f32, r: f32, start_angle: f32, sweep_angle: f32, moveto: bool, builder: &mut PathBuilder) {
    let segments_count = (sweep_angle.abs() / MAX_SEGMENT_ANGLE).ceil() as usize;
    let segment_sweep = sweep_angle / segments_count as f32;

    let lr = (4.0 / 3.0) * (segment_sweep / 4.0).tan() * r;

    let mut cur_angle = start_angle;
    if moveto {
        let start_x = cx + r * cur_angle.cos();
        let start_y = cy + r * cur_angle.sin();
        builder.move_to(start_x, -start_y);
    }

    for _ in 0..segments_count {
        let next_angle = cur_angle + segment_sweep;

        let (sin_curr, cos_curr) = cur_angle.sin_cos();
        let (sin_next, cos_next) = next_angle.sin_cos();

        let x0 = cx + r * cos_curr;
        let y0 = cy + r * sin_curr;

        let x = cx + r * cos_next;
        let y = cy + r * sin_next;

        let x1 = x0 - lr * sin_curr;
        let y1 = y0 + lr * cos_curr;

        let x2 = x + lr * sin_next;
        let y2 = y - lr * cos_next;

        builder.cubic_to(x1, -y1, x2, -y2, x, -y);

        cur_angle = next_angle;
    }
}

fn arc_path(cx: f32, cy: f32, r: f32, start_angle: f32, end_angle: f32) -> anyhow::Result<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let sweep_angle = if start_angle <= end_angle {
        end_angle - start_angle
    } else {
        end_angle + std::f32::consts::TAU - start_angle
    };
    arc_draw(cx, cy, r, start_angle, sweep_angle, true, &mut pb);
    pb.finish().ok_or(anyhow::anyhow!("Path is empty"))
}

fn bulge_draw(x1: f32, y1: f32, x2: f32, y2: f32, bulge: f32, builder: &mut PathBuilder) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = dx.hypot(dy);

    let cx = (x1 + x2) * 0.5 - dy * (1.0 - bulge * bulge) / (4.0 * bulge);
    let cy = (y1 + y2) * 0.5 + dx * (1.0 - bulge * bulge) / (4.0 * bulge);
    let radius = len * (1.0 + bulge * bulge) / (4.0 * bulge.abs());

    let sweep_angle = 4.0 * bulge.atan();
    let start_angle = f32::atan2(y1 - cy, x1 - cx);

    arc_draw(cx, cy, radius, start_angle, sweep_angle, false, builder);
}

fn process_entities<'a, L>(list: L, blocks: &HashMap<String, FinishedBlock>) -> anyhow::Result<DrawingBlock>
where
    L: Iterator<Item = &'a Entity>,
{
    let mut drawing = DrawingBlock::new();

    for e in list {
        let pb = drawing.request_path(e.common.color_24_bit);
        let primary = is_primary_color(e.common.color_24_bit);
        match e.specific {
            EntityType::Circle(ref circle) => {
                pb.push_circle(circle.center.x as f32, -circle.center.y as f32, circle.radius as f32);
            }

            EntityType::Line(ref line) => {
                let x1 = line.p1.x as f32;
                let y1 = -line.p1.y as f32;
                let x2 = line.p2.x as f32;
                let y2 = -line.p2.y as f32;
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);
                if primary {
                    drawing.open_ends.push(Point2D::new(x1, y1));
                    drawing.open_ends.push(Point2D::new(x2, y2));
                }
            }

            EntityType::Arc(ref arc) => {
                let cx = arc.center.x as f32;
                let cy = arc.center.y as f32;
                let r = arc.radius as f32;
                let start_angle = arc.start_angle.to_radians() as f32;
                let end_angle = arc.end_angle.to_radians() as f32;
                pb.push_path(&arc_path(cx, cy, r, start_angle, end_angle)?);
                if primary {
                    let start_point = Point2D::new(cx + r * start_angle.cos(), cy + r * start_angle.sin());
                    let end_point = Point2D::new(cx + r * end_angle.cos(), cy + r * end_angle.sin());
                    drawing.open_ends.push(start_point);
                    drawing.open_ends.push(end_point);
                }
            }

            EntityType::Ellipse(ref ellipse) => {
                let radius = (ellipse.major_axis.x * ellipse.major_axis.x + ellipse.major_axis.y * ellipse.major_axis.y)
                    .sqrt() as f32;
                let angle = {
                    let res = (ellipse.major_axis.x as f32 / radius).acos();
                    if ellipse.major_axis.y > 0.0 { -res } else { res }
                };

                let transform = Transform::from_scale(1.0, ellipse.minor_axis_ratio as f32)
                    .post_rotate(angle.to_degrees())
                    .post_translate(ellipse.center.x as f32, -ellipse.center.y as f32);

                if ellipse.start_parameter.abs() < EPSILON && (ellipse.end_parameter - TAU).abs() < EPSILON {
                    // Full ellipse
                    let path = PathBuilder::from_circle(0.0, 0.0, radius)
                        .ok_or(anyhow::anyhow!("Path is empty"))?
                        .transform(transform)
                        .ok_or(anyhow::anyhow!("Cant transform an ellipse"))?;
                    pb.push_path(&path);
                } else {
                    // Elliptic arc
                    let start_angle = ellipse.start_parameter as f32;
                    let end_angle = ellipse.end_parameter as f32;
                    let path = arc_path(0.0, 0.0, radius, start_angle, end_angle)?
                        .transform(transform)
                        .ok_or(anyhow::anyhow!("Cant transform an ellipse"))?;
                    pb.push_path(&path);
                    if primary {
                        let start_point = Point2D::new(radius * start_angle.cos(), radius * start_angle.sin());
                        let end_point = Point2D::new(radius * end_angle.cos(), radius * end_angle.sin());
                        drawing.open_ends.push(start_point.transform(transform));
                        drawing.open_ends.push(end_point.transform(transform));
                    }
                };
            }

            EntityType::LwPolyline(ref polyline) => {
                if !polyline.vertices.is_empty() {
                    let mut line = PathBuilder::new();
                    line.move_to(polyline.vertices[0].x as f32, -polyline.vertices[0].y as f32);
                    for point in polyline.vertices.windows(2) {
                        let p1 = point[0];
                        let p2 = point[1];
                        if p1.bulge.abs() > EPSILON {
                            bulge_draw(
                                p1.x as f32,
                                p1.y as f32,
                                p2.x as f32,
                                p2.y as f32,
                                p1.bulge as f32,
                                &mut line,
                            );
                        } else {
                            line.line_to(p2.x as f32, -p2.y as f32);
                        }
                    }
                    let p1 = polyline.vertices.last().unwrap();
                    let p2 = polyline.vertices.first().unwrap();
                    if polyline.flags & 1 > 0 {
                        // Closed polyline
                        if p1.bulge.abs() > EPSILON {
                            bulge_draw(
                                p1.x as f32,
                                p1.y as f32,
                                p2.x as f32,
                                p2.y as f32,
                                p1.bulge as f32,
                                &mut line,
                            );
                        } else {
                            line.close();
                        }
                    }

                    pb.push_path(&line.finish().ok_or(anyhow::anyhow!("Path is empty"))?);

                    if primary && polyline.flags & 1 == 0 {
                        // Open polyline
                        drawing.open_ends.push(Point2D::new(p1.x as f32, p1.y as f32));
                        drawing.open_ends.push(Point2D::new(p2.x as f32, p2.y as f32));
                    }
                }
            }

            EntityType::Spline(ref spline) => {
                pb.push_path(&curve_path(spline).unwrap());
                if primary && spline.flags & 1 == 0 {
                    //Open spline
                    let p1 = spline.control_points.first().unwrap();
                    let p2 = spline.control_points.last().unwrap();
                    drawing.open_ends.push(Point2D::new(p1.x as f32, p1.y as f32));
                    drawing.open_ends.push(Point2D::new(p2.x as f32, p2.y as f32));
                }
            }

            EntityType::Insert(ref insert) => {
                if let Some(block) = blocks.get(&insert.name) {
                    for col in 0..insert.column_count {
                        for row in 0..insert.row_count {
                            let x = (insert.location.x + col as f64 * insert.column_spacing) as f32;
                            let y = (insert.location.y + row as f64 * insert.row_spacing) as f32;
                            let transform =
                                Transform::from_scale(insert.x_scale_factor as f32, insert.y_scale_factor as f32)
                                    .post_translate(x, y);

                            for (color, path) in &block.layers {
                                drawing.request_path(*color).push_path(
                                    &path
                                        .clone()
                                        .transform(transform)
                                        .ok_or(anyhow::anyhow!("Cannot transform block"))?,
                                );
                            }

                            for point in &block.open_ends {
                                drawing.open_ends.push(point.transform(transform));
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    Ok(drawing)
}

pub fn load_dxf(filename: impl AsRef<Path>) -> anyhow::Result<tiny_skia::Path> {
    let drawing = Drawing::load_file(filename).unwrap();

    let mut blocks = HashMap::new();

    for block in drawing.blocks() {
        if !block.entities.is_empty() {
            blocks.insert(
                block.name.clone(),
                process_entities(block.entities.iter(), &blocks)?.finish()?,
            );
        }
    }

    let main_block = process_entities(drawing.entities(), &blocks)?;

    let mut pb = PathBuilder::new();
    for drawing in main_block.layers.values() {
        pb.push_path(&drawing.clone().finish().unwrap());
    }
    pb.finish().ok_or(anyhow::anyhow!("Path is empty"))
}
