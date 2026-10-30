use dxf::{
    Drawing,
    entities::{Entity, EntityType},
};
use std::{collections::HashMap, f64::consts::TAU, io::Cursor, path::Path, vec};
use tiny_skia::{PathBuilder, Transform};

use crate::{
    config::CONFIG,
    curves::curve_path,
    errors::{default_rect, finish_path},
    geometry::{EPSILON64, Point2D},
    pointerrors::{PointErrors, collect_errors},
};

const MAX_SEGMENT_ANGLE: f32 = std::f32::consts::FRAC_PI_2;

const DEFAULT_COLORS: [i32; 256] = [
    0x000000, 0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF, 0x0000FF, 0xFF00FF, 0x000000, 0x414141, 0x808080, 0xFF0000,
    0xFFAAAA, 0xBD0000, 0xBD7E7E, 0x810000, 0x815656, 0x680000, 0x684545, 0x4F0000, 0x4F3535, 0xFF3F00, 0xFFBFAA,
    0xBD2E00, 0xBD8D7E, 0x811F00, 0x816056, 0x681900, 0x684E45, 0x4F1300, 0x4F3B35, 0xFF7F00, 0xFFD4AA, 0xBD5E00,
    0xBD9D7E, 0x814000, 0x816B56, 0x683400, 0x685645, 0x4F2700, 0x4F4235, 0xFFBF00, 0xFFEAAA, 0xBD8D00, 0xBDAD7E,
    0x816000, 0x817656, 0x684E00, 0x685F45, 0x4F3B00, 0x4F4935, 0xFFFF00, 0xFFFFAA, 0xBDBD00, 0xBDBD7E, 0x818100,
    0x818156, 0x686800, 0x686845, 0x4F4F00, 0x4F4F35, 0xBFFF00, 0xEAFFAA, 0x8DBD00, 0xADBD7E, 0x608100, 0x768156,
    0x4E6800, 0x5F6845, 0x3B4F00, 0x494F35, 0x7FFF00, 0xD4FFAA, 0x5EBD00, 0x9DBD7E, 0x408100, 0x6B8156, 0x346800,
    0x566845, 0x274F00, 0x424F35, 0x3FFF00, 0xBFFFAA, 0x2EBD00, 0x8DBD7E, 0x1F8100, 0x608156, 0x196800, 0x4E6845,
    0x134F00, 0x3B4F35, 0x00FF00, 0xAAFFAA, 0x00BD00, 0x7EBD7E, 0x008100, 0x568156, 0x006800, 0x456845, 0x004F00,
    0x354F35, 0x00FF3F, 0xAAFFBF, 0x00BD2E, 0x7EBD8D, 0x00811F, 0x568160, 0x006819, 0x45684E, 0x004F13, 0x354F3B,
    0x00FF7F, 0xAAFFD4, 0x00BD5E, 0x7EBD9D, 0x008140, 0x56816B, 0x006834, 0x456856, 0x004F27, 0x354F42, 0x00FFBF,
    0xAAFFEA, 0x00BD8D, 0x7EBDAD, 0x008160, 0x568176, 0x00684E, 0x45685F, 0x004F3B, 0x354F49, 0x00FFFF, 0xAAFFFF,
    0x00BDBD, 0x7EBDBD, 0x008181, 0x568181, 0x006868, 0x456868, 0x004F4F, 0x354F4F, 0x00BFFF, 0xAAEAFF, 0x008DBD,
    0x7EADBD, 0x006081, 0x567681, 0x004E68, 0x455F68, 0x003B4F, 0x35494F, 0x007FFF, 0xAAD4FF, 0x005EBD, 0x7E9DBD,
    0x004081, 0x566B81, 0x003468, 0x455668, 0x00274F, 0x35424F, 0x003FFF, 0xAABFFF, 0x002EBD, 0x7E8DBD, 0x001F81,
    0x566081, 0x001968, 0x454E68, 0x00134F, 0x353B4F, 0x0000FF, 0xAAAAFF, 0x0000BD, 0x7E7EBD, 0x000081, 0x565681,
    0x000068, 0x454568, 0x00004F, 0x35354F, 0x3F00FF, 0xBFAAFF, 0x2E00BD, 0x8D7EBD, 0x1F0081, 0x605681, 0x190068,
    0x4E4568, 0x13004F, 0x3B354F, 0x7F00FF, 0xD4AAFF, 0x5E00BD, 0x9D7EBD, 0x400081, 0x6B5681, 0x340068, 0x564568,
    0x27004F, 0x42354F, 0xBF00FF, 0xEAAAFF, 0x8D00BD, 0xAD7EBD, 0x600081, 0x765681, 0x4E0068, 0x5F4568, 0x3B004F,
    0x49354F, 0xFF00FF, 0xFFAAFF, 0xBD00BD, 0xBD7EBD, 0x810081, 0x815681, 0x680068, 0x684568, 0x4F004F, 0x4F354F,
    0xFF00BF, 0xFFAAEA, 0xBD008D, 0xBD7EAD, 0x810060, 0x815676, 0x68004E, 0x68455F, 0x4F003B, 0x4F3549, 0xFF007F,
    0xFFAAD4, 0xBD005E, 0xBD7E9D, 0x810040, 0x81566B, 0x680034, 0x684556, 0x4F0027, 0x4F3542, 0xFF003F, 0xFFAABF,
    0xBD002E, 0xBD7E8D, 0x81001F, 0x815660, 0x680019, 0x68454E, 0x4F0013, 0x4F353B, 0x333333, 0x505050, 0x696969,
    0x828282, 0xBEBEBE, 0xFFFFFF,
];

pub const LINEWIDTH_BIAS: f32 = 0.05;

#[derive(Debug, Clone, Copy)]
pub enum LineType {
    Main,
    Cuts,
    Marks,
    Other,
}

#[derive(Default)]
struct DrawingLayer {
    builder: tiny_skia::PathBuilder,
    open_ends: Vec<Point2D>,
}

struct DrawingLayers(HashMap<i32, DrawingLayer>);

struct BlockLayer {
    path: tiny_skia::Path,
    open_ends: Vec<Point2D>,
}
struct DrawingBlock {
    layers: Vec<(i32, BlockLayer)>,
    by_layer: Option<BlockLayer>,
    by_block: Option<BlockLayer>,
}

pub struct BlueprintLayer<'a> {
    pub drawing: tiny_skia::Path,
    pub color: tiny_skia::Paint<'a>,
    pub line_width: f32,
    pub order: u32,
    pub hint_color: i32,
    pub hint_type: LineType,
}
pub struct Blueprint<'a> {
    pub layers: Vec<BlueprintLayer<'a>>,
    pub bounds: tiny_skia::Rect,
    pub errors: PointErrors,
}

impl DrawingLayers {
    fn new() -> DrawingLayers {
        DrawingLayers(HashMap::new())
    }

    fn request_path(&mut self, color: i32) -> &mut DrawingLayer {
        self.0.entry(color).or_default();
        self.0.get_mut(&color).unwrap() // existence of the entry is ensured by the previous line
    }
}

impl DrawingLayer {
    fn finish(self) -> Option<BlockLayer> {
        match self.builder.finish() {
            Some(path) => Some(BlockLayer {
                path,
                open_ends: self.open_ends,
            }),
            None => None,
        }
    }
}

impl<'a> Default for Blueprint<'a> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            bounds: default_rect(),
            errors: Default::default(),
        }
    }
}

fn get_line_type(color: i32) -> LineType {
    if CONFIG.colors.main.contains(&color) {
        LineType::Main
    } else if CONFIG.colors.cuts.contains(&color) {
        LineType::Cuts
    } else if CONFIG.colors.marks.contains(&color) {
        LineType::Marks
    } else {
        LineType::Other
    }
}

pub fn arc_draw(cx: f32, cy: f32, r: f32, start_angle: f32, sweep_angle: f32, moveto: bool, builder: &mut PathBuilder) {
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
    finish_path(pb)
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

fn transfer_layer(dst: &mut DrawingLayer, src: &BlockLayer, transform: Transform) -> anyhow::Result<()> {
    dst.builder.push_path(
        &src.path
            .clone()
            .transform(transform)
            .ok_or(anyhow::anyhow!("Cannot transform block"))?,
    );

    dst.open_ends
        .extend(src.open_ends.iter().map(|p| p.transform(transform)));

    Ok(())
}

fn process_entities<'a, L>(
    list: L,
    blocks: &HashMap<String, DrawingBlock>,
    layer_colors: &HashMap<String, i32>,
    inside_block: bool,
) -> anyhow::Result<DrawingBlock>
where
    L: Iterator<Item = &'a Entity>,
{
    let mut drawing_layers = DrawingLayers::new();
    let mut by_layer = DrawingLayer::default();
    let mut by_block = DrawingLayer::default();

    for e in list {
        let dl = {
            if e.common.color_24_bit > 0 {
                drawing_layers.request_path(e.common.color_24_bit)
            } else if e.common.color.is_by_block() {
                if inside_block {
                    &mut by_block
                } else {
                    anyhow::bail!("BYBLOCK color outside of block")
                }
            } else if e.common.color.is_by_layer() {
                if inside_block {
                    &mut by_layer
                } else if let Some(lcolor) = layer_colors.get(&e.common.layer) {
                    drawing_layers.request_path(*lcolor)
                } else {
                    drawing_layers.request_path(0x000000)
                }
            } else if let Some(index) = e.common.color.index() {
                drawing_layers.request_path(DEFAULT_COLORS[index as usize])
            } else {
                drawing_layers.request_path(0x000000)
            }
        };
        let pb = &mut dl.builder;
        let open_ends = &mut dl.open_ends;

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

                open_ends.push(Point2D::new(x1, y1));
                open_ends.push(Point2D::new(x2, y2));
            }

            EntityType::Arc(ref arc) => {
                let cx = arc.center.x as f32;
                let cy = arc.center.y as f32;
                let r = arc.radius as f32;
                let start_angle = arc.start_angle.to_radians() as f32;
                let end_angle = arc.end_angle.to_radians() as f32;
                pb.push_path(&arc_path(cx, cy, r, start_angle, end_angle)?);

                let start_point = Point2D::new(cx + r * start_angle.cos(), -(cy + r * start_angle.sin()));
                let end_point = Point2D::new(cx + r * end_angle.cos(), -(cy + r * end_angle.sin()));
                open_ends.push(start_point);
                open_ends.push(end_point);
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

                if ellipse.start_parameter.abs() < EPSILON64 && (ellipse.end_parameter - TAU).abs() < EPSILON64 {
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

                    let start_point = Point2D::new(radius * start_angle.cos(), -radius * start_angle.sin());
                    let end_point = Point2D::new(radius * end_angle.cos(), -radius * end_angle.sin());
                    open_ends.push(start_point.transform(transform));
                    open_ends.push(end_point.transform(transform));
                };
            }

            EntityType::LwPolyline(ref polyline) => {
                if !polyline.vertices.is_empty() {
                    let mut line = PathBuilder::new();
                    line.move_to(polyline.vertices[0].x as f32, -polyline.vertices[0].y as f32);
                    for point in polyline.vertices.windows(2) {
                        let p1 = point[0];
                        let p2 = point[1];
                        if p1.bulge.abs() > EPSILON64 {
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
                    let p1 = polyline.vertices.last().unwrap(); // polyline have at least one entry
                    let p2 = polyline.vertices.first().unwrap();
                    if polyline.flags & 1 > 0 {
                        // Closed polyline
                        if p1.bulge.abs() > EPSILON64 {
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

                    pb.push_path(&finish_path(line)?);

                    if polyline.flags & 1 == 0 {
                        // Open polyline
                        open_ends.push(Point2D::new(p1.x as f32, -p1.y as f32));
                        open_ends.push(Point2D::new(p2.x as f32, -p2.y as f32));
                    }
                }
            }

            EntityType::Spline(ref spline) => {
                pb.push_path(&curve_path(spline)?);
                if spline.control_points.len() > 1 && spline.flags & 1 == 0 {
                    //Open spline
                    let p1 = spline.control_points.first().unwrap(); // spline have at least 2 points
                    let p2 = spline.control_points.last().unwrap();
                    let start_point = Point2D::new(p1.x as f32, -p1.y as f32);
                    let end_point = Point2D::new(p2.x as f32, -p2.y as f32);
                    open_ends.push(start_point);
                    open_ends.push(end_point);
                }
            }

            EntityType::Insert(ref insert) => {
                if let Some(block) = blocks.get(&insert.name) {
                    if let Some(src) = block.by_block.as_ref() {
                        for col in 0..insert.column_count {
                            for row in 0..insert.row_count {
                                let x = (insert.location.x + col as f64 * insert.column_spacing) as f32;
                                let y = (insert.location.y + row as f64 * insert.row_spacing) as f32;
                                let transform =
                                    Transform::from_scale(insert.x_scale_factor as f32, insert.y_scale_factor as f32)
                                        .post_translate(x, y);
                                transfer_layer(dl, src, transform)?;
                            }
                        }
                    }

                    for col in 0..insert.column_count {
                        for row in 0..insert.row_count {
                            let x = (insert.location.x + col as f64 * insert.column_spacing) as f32;
                            let y = (insert.location.y + row as f64 * insert.row_spacing) as f32;
                            let transform =
                                Transform::from_scale(insert.x_scale_factor as f32, insert.y_scale_factor as f32)
                                    .post_translate(x, y);

                            for (color, block_layer) in &block.layers {
                                let drawing_layer = drawing_layers.request_path(*color);
                                transfer_layer(drawing_layer, block_layer, transform)?;
                            }

                            if let Some(src) = block.by_layer.as_ref() {
                                let by_layer_drawing = if let Some(lcolor) = layer_colors.get(&e.common.layer) {
                                    drawing_layers.request_path(*lcolor)
                                } else {
                                    drawing_layers.request_path(0x000000)
                                };
                                transfer_layer(by_layer_drawing, src, transform)?;
                            }
                        }
                    }
                }
            }

            _ => (),
        }
    }

    let mut layers = vec![];
    for (color, builder) in drawing_layers.0 {
        if let Some(path) = builder.finish() {
            layers.push((color, path));
        }
    }
    Ok(DrawingBlock {
        layers,
        by_layer: by_layer.finish(),
        by_block: by_block.finish(),
    })
}

pub fn load_dxf<'a>(filename: impl AsRef<Path>) -> anyhow::Result<Blueprint<'a>> {
    let drawing = Drawing::load_file(filename)?;
    parse_dxf(drawing)
}

pub fn decode_dxf<'a>(data: &[u8]) -> anyhow::Result<Blueprint<'a>> {
    let mut cursor = Cursor::new(data);
    let drawing = Drawing::load(&mut cursor)?;
    parse_dxf(drawing)
}

pub fn parse_dxf<'a>(drawing: Drawing) -> anyhow::Result<Blueprint<'a>> {
    let layer_colors: HashMap<String, i32> = drawing
        .layers()
        .map(|l| (l.name.clone(), DEFAULT_COLORS[l.color.index().unwrap_or(7) as usize]))
        .collect();

    let mut blocks = HashMap::new();

    for block in drawing.blocks() {
        if !block.entities.is_empty() {
            blocks.insert(
                block.name.clone(),
                process_entities(block.entities.iter(), &blocks, &layer_colors, true)?,
            );
        }
    }

    let main_block = process_entities(drawing.entities(), &blocks, &layer_colors, false)?;

    let mut layers = vec![];
    let mut bounds: Option<tiny_skia::Rect> = None;
    let mut open_ends = vec![];

    for (color, path) in main_block.layers {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(r, g, b, 255);
        paint.anti_alias = CONFIG.anti_aliasing;

        let layer_bounds = path.path.bounds();

        let line_type = get_line_type(color);
        let (line_width, order) = match line_type {
            LineType::Main => {
                open_ends.extend(path.open_ends);
                (CONFIG.line_widths.main, 4)
            }
            LineType::Cuts => (CONFIG.line_widths.thick, 3),
            LineType::Marks => (CONFIG.line_widths.thin, 2),
            LineType::Other => (CONFIG.line_widths.thin, 1),
        };

        layers.push(BlueprintLayer {
            drawing: path.path,
            color: paint,
            line_width,
            order,
            hint_color: color,
            hint_type: line_type,
        });

        if let Some(some_bounds) = bounds.as_mut() {
            bounds = some_bounds.join(&layer_bounds);
        } else {
            bounds = Some(layer_bounds);
        }
    }

    layers.sort_by_key(|l| l.order);

    let errors = PointErrors::new(collect_errors(&open_ends));

    Ok(Blueprint {
        layers,
        bounds: bounds.ok_or(anyhow::anyhow!("Bounds are empty"))?,
        errors,
    })
}
