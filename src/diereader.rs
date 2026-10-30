use std::{
    f32::consts::TAU,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::bail;
use dxf::Drawing;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use tiny_skia::{PathBuilder, Rect};

use crate::{
    blueprint::{self, Blueprint, BlueprintLayer, arc_draw},
    config::CONFIG,
    errors::default_rect,
    geometry::{EPSILON, Point2D},
    pointerrors::PointErrors,
};

const DEFAULT_MAIN_COLOR: i32 = 0x007FFF;
const DEFAULT_CUTS_COLOR: i32 = 0x008181;
const DEFAULT_MARKS_COLOR: i32 = 0x000000;

const DEFAULT_MAIN_COLOR_INDEX: u8 = 150;
const DEFAULT_CUTS_COLOR_INDEX: u8 = 136;
const DEFAULT_MARKS_COLOR_INDEX: u8 = 7;

pub const DXF_LINEWEIGHT_MAIN: i16 = 60;
pub const DXF_LINEWEIGHT_THICK: i16 = 100;
pub const DXF_LINEWEIGHT_OTHER: i16 = 20;

pub const DXF_LINEWIDTH_MAIN: f64 = 0.6;
pub const DXF_LINEWIDTH_THICK: f64 = 1.0;
pub const DXF_LINEWIDTH_OTHER: f64 = 0.2;

#[derive(Debug, Clone, Copy)]
struct DieArc {
    start: usize,
    end: usize,
    center: Point2D,
    direction: bool,
}

#[derive(Debug, Clone, Copy)]
enum DieCommand {
    NumOfPoints(bool, usize),
    Point(bool, Point2D),
    NumOfArcs(bool, usize),
    Arc(bool, DieArc),
    NumOfSign(usize),
    Sign(Point2D, Point2D),
    NumOfCuts(usize),
    EndOfFile,
}

fn get_intparam(list: &[&str], index: usize) -> usize {
    match list.get(index) {
        Some(param) => param.parse().unwrap_or_default(),
        None => 0,
    }
}

fn get_floatparam(list: &[&str], index: usize) -> f32 {
    match list.get(index) {
        Some(param) => param.parse().unwrap_or_default(),
        None => 0.0,
    }
}

struct DieFile {
    reader: BufReader<DecodeReaderBytes<File, Vec<u8>>>,
    line: String,
    command: DieCommand,
}

impl DieFile {
    fn open(filename: impl AsRef<Path>) -> anyhow::Result<DieFile> {
        let file = File::open(filename)?;
        let reader = BufReader::new(DecodeReaderBytesBuilder::new().encoding(Some(WINDOWS_1252)).build(file));

        Ok(DieFile {
            reader,
            line: String::new(),
            command: DieCommand::EndOfFile,
        })
    }

    fn next(&mut self) {
        loop {
            let bytes_read = self.reader.read_line(&mut self.line).unwrap_or(0);

            if bytes_read == 0 {
                self.command = DieCommand::EndOfFile;
                return;
            }

            let Some((command, params)) = self.line.split_once('=') else {
                self.line.clear();
                continue;
            };

            let command = command.trim();
            let params: Vec<&str> = params
                .trim()
                .split(' ')
                .filter_map(|param| {
                    let res = param.trim();
                    if res.is_empty() { None } else { Some(res) }
                })
                .collect();

            let command = match command {
                "num_of_points" => DieCommand::NumOfPoints(true, get_intparam(&params, 0)),
                "point" => DieCommand::Point(
                    true,
                    Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1)),
                ),
                "num_of_c_arcs" => DieCommand::NumOfArcs(true, get_intparam(&params, 0)),
                "c_arc" => DieCommand::Arc(
                    true,
                    DieArc {
                        start: get_intparam(&params, 0),
                        end: get_intparam(&params, 1),
                        direction: get_intparam(&params, 2) > 0,
                        center: Point2D::new(get_floatparam(&params, 3), get_floatparam(&params, 4)),
                    },
                ),
                "num_of_sign" => DieCommand::NumOfSign(get_intparam(&params, 0)),
                "sign" => DieCommand::Sign(
                    Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1)),
                    Point2D::new(get_floatparam(&params, 2), get_floatparam(&params, 3)),
                ),
                "num_of_cuts" => DieCommand::NumOfCuts(get_intparam(&params, 0)),
                "num_of_cpoints" => DieCommand::NumOfPoints(false, get_intparam(&params, 0)),
                "cpoint" => DieCommand::Point(
                    false,
                    Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1)),
                ),
                "num_of_cut_arcs" => DieCommand::NumOfArcs(false, get_intparam(&params, 0)),
                "cut_arc" => DieCommand::Arc(
                    false,
                    DieArc {
                        start: get_intparam(&params, 0),
                        end: get_intparam(&params, 1),
                        direction: get_intparam(&params, 2) > 0,
                        center: Point2D::new(get_floatparam(&params, 3), get_floatparam(&params, 4)),
                    },
                ),
                _ => {
                    self.line.clear();
                    continue;
                }
            };

            self.line.clear();
            self.command = command;
            return;
        }
    }
}

pub fn new_layer<'a>(
    path: tiny_skia::Path,
    color: i32,
    width: f32,
    line_type: blueprint::LineType,
) -> BlueprintLayer<'a> {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint.anti_alias = CONFIG.anti_aliasing;

    BlueprintLayer {
        drawing: path,
        color: paint,
        line_width: width,
        order: 0,
        hint_color: color,
        hint_type: line_type,
    }
}

fn arc_draw_by_points(center: Point2D, start: Point2D, end: Point2D, direction: bool, pb: &mut PathBuilder) {
    let radius = (center.dist2(start).sqrt() + center.dist2(end).sqrt()) / 2.0;
    if start.dist2(end) < EPSILON {
        pb.push_circle(center.x, -center.y, radius);
        pb.move_to(end.x, -end.y);
    } else {
        let start_angle = f32::atan2(start.y - center.y, start.x - center.x);
        let end_angle = f32::atan2(end.y - center.y, end.x - center.x);

        let mut sweep_angle = end_angle - start_angle;
        if direction && sweep_angle < 0.0 {
            sweep_angle += TAU;
        }
        if !direction && sweep_angle > 0.0 {
            sweep_angle -= TAU;
        }
        arc_draw(center.x, center.y, radius, start_angle, sweep_angle, false, pb);
    }
}

fn parse_lines(file: &mut DieFile, pb: &mut PathBuilder) -> Option<Rect> {
    let DieCommand::NumOfSign(count) = file.command else {
        return None;
    };

    let mut minx = f32::MAX;
    let mut maxx = f32::MIN;
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;

    file.next();

    // skip
    loop {
        if let DieCommand::Sign(_, _) = file.command {
            break;
        } else {
            file.next();
        }
    }

    for _ in 0..count {
        let DieCommand::Sign(p1, p2) = file.command else {
            break;
        };

        pb.move_to(p1.x, -p1.y);
        pb.line_to(p2.x, -p2.y);

        minx = minx.min(p1.x);
        maxx = maxx.max(p1.x);
        miny = miny.min(-p1.y);
        maxy = maxy.max(-p1.y);

        minx = minx.min(p2.x);
        maxx = maxx.max(p2.x);
        miny = miny.min(-p2.y);
        maxy = maxy.max(-p2.y);

        file.next();
    }

    // skip excess
    while let DieCommand::Sign(_, _) = file.command {
        file.next();
    }

    Rect::from_ltrb(minx, miny, maxx, maxy)
}

fn parse_contour(file: &mut DieFile, pb: &mut PathBuilder, outer: bool) -> anyhow::Result<Rect> {
    if let DieCommand::NumOfPoints(is_outer, count) = file.command
        && is_outer == outer
    {
        let mut minx = f32::MAX;
        let mut maxx = f32::MIN;
        let mut miny = f32::MAX;
        let mut maxy = f32::MIN;

        let mut points = vec![];
        let mut arcs = vec![];

        file.next();

        // skip
        loop {
            if let DieCommand::Point(_, _) = file.command {
                break;
            } else {
                file.next();
            }
        }

        for _ in 0..count {
            if let DieCommand::Point(is_outer, point) = file.command
                && is_outer == outer
            {
                points.push(point);

                minx = minx.min(point.x);
                maxx = maxx.max(point.x);
                miny = miny.min(-point.y);
                maxy = maxy.max(-point.y);
                file.next();
            } else {
                match file.command {
                    DieCommand::NumOfArcs(_, _) => {
                        bail!("Insufficient number of vertices")
                    }
                    _ => break,
                }
            };
        }

        // skip excess
        while let DieCommand::Point(_, _) = file.command {
            file.next();
        }

        if let DieCommand::NumOfArcs(is_outer, count) = file.command
            && is_outer == outer
        {
            file.next();

            loop {
                if let DieCommand::Arc(_, _) = file.command {
                    break;
                } else {
                    file.next();
                }
            }

            for _ in 0..count {
                if let DieCommand::Arc(is_outer, arc) = file.command
                    && is_outer == outer
                {
                    arcs.push(arc);
                    file.next();
                } else {
                    break;
                };
            }

            // skip excess
            while let DieCommand::Arc(_, _) = file.command {
                file.next();
            }
        }

        if !points.is_empty() {
            pb.move_to(points[0].x, -points[0].y);
            if arcs.is_empty() {
                for pt in points.iter().skip(1) {
                    pb.line_to(pt.x, -pt.y);
                }
            } else {
                arcs.sort_by_key(|a| a.start);
                let mut cur_point = 0;
                let mut cur_arc = 0;
                while cur_point < points.len() - 1 && cur_arc < arcs.len() {
                    if cur_point == arcs[cur_arc].start {
                        let arc = arcs[cur_arc];
                        arc_draw_by_points(arc.center, points[arc.start], points[arc.end], arc.direction, pb);
                        cur_point = arc.end;
                        cur_arc += 1;
                    } else {
                        cur_point += 1;
                        let next = points[cur_point];
                        pb.line_to(next.x, -next.y);
                    }
                }
                if cur_point < points.len() - 1 {
                    for pt in points.iter().skip(cur_point) {
                        pb.line_to(pt.x, -pt.y);
                    }
                }
            }
        }

        Rect::from_ltrb(minx, miny, maxx, maxy).ok_or(anyhow::anyhow!("Wrong bounds"))
    } else {
        Ok(default_rect())
    }
}

fn parse_cuts(file: &mut DieFile, pb: &mut PathBuilder) -> anyhow::Result<Option<Rect>> {
    let mut bounds: Option<Rect> = None;

    let DieCommand::NumOfCuts(count) = file.command else {
        return Ok(None);
    };
    file.next();

    // skip
    loop {
        if let DieCommand::NumOfPoints(_, _) = file.command {
            break;
        } else {
            file.next();
        }
    }

    for _ in 0..count {
        let new_bounds = parse_contour(file, pb, false)?;
        if let Some(bounds) = bounds.as_mut() {
            bounds.join(&new_bounds);
        } else {
            bounds = Some(new_bounds)
        }
    }

    Ok(bounds)
}

pub fn load_die<'a>(filename: impl AsRef<Path>) -> anyhow::Result<Blueprint<'a>> {
    let mut file = DieFile::open(filename)?;

    let mut main_pb = PathBuilder::new();
    let mut cuts_pb = PathBuilder::new();
    let mut marks_pb = PathBuilder::new();

    let mut bounds: Option<Rect> = None;

    file.next();
    loop {
        match file.command {
            DieCommand::EndOfFile => break,
            DieCommand::NumOfSign(_) => {
                let new_bounds = parse_lines(&mut file, &mut marks_pb);
                if let Some(new_bounds) = new_bounds {
                    if let Some(total_bounds) = bounds.as_mut() {
                        total_bounds.join(&new_bounds);
                    } else {
                        bounds = Some(new_bounds)
                    }
                }
            }
            DieCommand::NumOfPoints(true, _) => {
                let new_bounds = parse_contour(&mut file, &mut main_pb, true)?;
                if let Some(total_bounds) = bounds.as_mut() {
                    total_bounds.join(&new_bounds);
                } else {
                    bounds = Some(new_bounds)
                }
            }
            DieCommand::NumOfCuts(_) => {
                let new_bounds = parse_cuts(&mut file, &mut cuts_pb)?;
                if let Some(new_bounds) = new_bounds {
                    if let Some(total_bounds) = bounds.as_mut() {
                        total_bounds.join(&new_bounds);
                    } else {
                        bounds = Some(new_bounds)
                    }
                }
            }
            _ => file.next(),
        }
    }

    let mut layers = vec![];

    if let Some(path) = marks_pb.finish() {
        layers.push(new_layer(
            path,
            DEFAULT_MARKS_COLOR,
            CONFIG.line_widths.thin,
            blueprint::LineType::Marks,
        ));
    }
    if let Some(path) = cuts_pb.finish() {
        layers.push(new_layer(
            path,
            DEFAULT_CUTS_COLOR,
            CONFIG.line_widths.thick,
            blueprint::LineType::Cuts,
        ));
    }
    if let Some(path) = main_pb.finish() {
        layers.push(new_layer(
            path,
            DEFAULT_MAIN_COLOR,
            CONFIG.line_widths.main,
            blueprint::LineType::Main,
        ));
    }

    Ok(Blueprint {
        layers,
        bounds: bounds.unwrap_or(default_rect()),
        errors: PointErrors::default(),
    })
}

// DIE -> DXF CONVERSION

fn set_line_type(entity: &mut dxf::entities::Entity, line_type: blueprint::LineType) {
    match line_type {
        blueprint::LineType::Main => {
            entity.common.color_24_bit = DEFAULT_MAIN_COLOR;
            entity.common.color = dxf::Color::from_index(DEFAULT_MAIN_COLOR_INDEX);
            entity.common.lineweight_enum_value = DXF_LINEWEIGHT_MAIN;
            entity.common.line_type_name = "CR_MAIN".into();
        }
        blueprint::LineType::Cuts => {
            entity.common.color_24_bit = DEFAULT_CUTS_COLOR;
            entity.common.color = dxf::Color::from_index(DEFAULT_CUTS_COLOR_INDEX);
            entity.common.lineweight_enum_value = DXF_LINEWEIGHT_THICK;
            entity.common.line_type_name = "CR_THICK".into();
        }
        blueprint::LineType::Marks => {
            entity.common.color_24_bit = DEFAULT_MARKS_COLOR;
            entity.common.color = dxf::Color::from_index(DEFAULT_MARKS_COLOR_INDEX);
            entity.common.lineweight_enum_value = DXF_LINEWEIGHT_OTHER;
            entity.common.line_type_name = "CR_THIN".into();
        }
        _ => (),
    }
}

fn get_line_width(line_type: blueprint::LineType) -> f64 {
    match line_type {
        blueprint::LineType::Main => DXF_LINEWIDTH_MAIN,
        blueprint::LineType::Cuts => DXF_LINEWIDTH_THICK,
        _ => DXF_LINEWIDTH_OTHER,
    }
}

#[derive(Clone, Copy)]
struct ConvVertex {
    point: Point2D,
    bulge: f32,
}

fn conv_circle(drawing: &mut Drawing, center: Point2D, radius: f32, line_type: blueprint::LineType) {
    let circle = dxf::entities::Circle::new(dxf::Point::new(center.x as f64, center.y as f64, 0.0), radius as f64);

    let mut circle = dxf::entities::Entity::new(dxf::entities::EntityType::Circle(circle));
    set_line_type(&mut circle, line_type);

    drawing.add_entity(circle);
}

fn conv_arc(
    drawing: &mut Drawing,
    center: Point2D,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    line_type: blueprint::LineType,
) {
    let arc = dxf::entities::Arc::new(
        dxf::Point::new(center.x as f64, center.y as f64, 0.0),
        radius as f64,
        (start_angle as f64).to_degrees(),
        (end_angle as f64).to_degrees(),
    );

    let mut arc = dxf::entities::Entity::new(dxf::entities::EntityType::Arc(arc));
    set_line_type(&mut arc, line_type);

    drawing.add_entity(arc);
}

fn conv_line(drawing: &mut Drawing, p1: Point2D, p2: Point2D, line_type: blueprint::LineType) {
    let line = dxf::entities::Line::new(
        dxf::Point::new(p1.x as f64, p1.y as f64, 0.0),
        dxf::Point::new(p2.x as f64, p2.y as f64, 0.0),
    );

    let mut line = dxf::entities::Entity::new(dxf::entities::EntityType::Line(line));
    set_line_type(&mut line, line_type);

    drawing.add_entity(line);
}

fn conv_polyline(drawing: &mut Drawing, vertices: &[ConvVertex], line_type: blueprint::LineType) {
    if vertices.len() < 2 {
        return;
    } else if vertices.len() == 2 {
        conv_line(drawing, vertices[0].point, vertices[1].point, line_type);
        return;
    }

    let mut vert_iter = vertices.iter();

    // vertices length is at least 2
    let closed = if vertices[0].point.dist2(vertices.last().unwrap().point) < EPSILON {
        vert_iter.next_back();
        true
    } else {
        false
    };

    let segments: Vec<dxf::LwPolylineVertex> = vert_iter
        .map(|vert| dxf::LwPolylineVertex {
            x: vert.point.x as f64,
            y: vert.point.y as f64,
            bulge: vert.bulge as f64,
            ..dxf::LwPolylineVertex::default()
        })
        .collect();

    let polyline = dxf::entities::LwPolyline {
        constant_width: get_line_width(line_type),
        vertices: segments,
        flags: if closed { 1 } else { 0 },
        ..dxf::entities::LwPolyline::default()
    };

    let mut polyline = dxf::entities::Entity::new(dxf::entities::EntityType::LwPolyline(polyline));
    set_line_type(&mut polyline, line_type);
    drawing.add_entity(polyline);
}

fn conv_parse_lines(file: &mut DieFile, drawing: &mut Drawing) {
    let DieCommand::NumOfSign(count) = file.command else {
        return;
    };

    let mut verts = vec![];
    let mut last = Point2D::default();

    file.next();

    // skip
    loop {
        if let DieCommand::Sign(_, _) = file.command {
            break;
        } else {
            file.next();
        }
    }

    for _ in 0..count {
        let DieCommand::Sign(p1, p2) = file.command else {
            break;
        };

        if verts.is_empty() {
            verts.push(ConvVertex { point: p1, bulge: 0.0 });
            verts.push(ConvVertex { point: p2, bulge: 0.0 });
            last = p2;
        } else if last.dist2(p1) < EPSILON {
            verts.push(ConvVertex { point: p2, bulge: 0.0 });
            last = p2;
        } else if last.dist2(p2) < EPSILON {
            verts.push(ConvVertex { point: p1, bulge: 0.0 });
            last = p1;
        } else {
            conv_polyline(drawing, &verts, blueprint::LineType::Marks);
            verts.clear();
            verts.push(ConvVertex { point: p1, bulge: 0.0 });
            verts.push(ConvVertex { point: p2, bulge: 0.0 });
            last = p2;
        }

        file.next();
    }
    conv_polyline(drawing, &verts, blueprint::LineType::Marks);

    // skip excess
    while let DieCommand::Sign(_, _) = file.command {
        file.next();
    }
}

fn conv_arc_by_points(
    drawing: &mut Drawing,
    center: Point2D,
    start: Point2D,
    end: Point2D,
    direction: bool,
    line_type: blueprint::LineType,
) {
    let radius = (center.dist2(start).sqrt() + center.dist2(end).sqrt()) / 2.0;
    if start.dist2(end) < EPSILON {
        conv_circle(drawing, center, radius, line_type);
    } else {
        let start_angle = f32::atan2(start.y - center.y, start.x - center.x);
        let end_angle = f32::atan2(end.y - center.y, end.x - center.x);

        let mut sweep_angle = end_angle - start_angle;
        if direction && sweep_angle < 0.0 {
            sweep_angle += TAU;
        }
        if !direction && sweep_angle > 0.0 {
            sweep_angle -= TAU;
        }
        conv_arc(
            drawing,
            center,
            radius,
            start_angle,
            start_angle + sweep_angle,
            line_type,
        );
    }
}

fn conv_get_bulge(center: Point2D, start: Point2D, end: Point2D, direction: bool) -> f32 {
    let start_angle = f32::atan2(start.y - center.y, start.x - center.x);
    let end_angle = f32::atan2(end.y - center.y, end.x - center.x);

    let mut sweep_angle = end_angle - start_angle;
    if direction && sweep_angle < 0.0 {
        sweep_angle += TAU;
    };
    if !direction && sweep_angle > 0.0 {
        sweep_angle -= TAU;
    };

    (sweep_angle / 4.0).tan()
}

fn conv_parse_contour(file: &mut DieFile, drawing: &mut Drawing, outer: bool) -> anyhow::Result<()> {
    if let DieCommand::NumOfPoints(is_outer, count) = file.command
        && is_outer == outer
    {
        let line_type = if outer {
            blueprint::LineType::Main
        } else {
            blueprint::LineType::Cuts
        };

        let mut points = vec![];
        let mut arcs = vec![];

        file.next();

        // skip
        loop {
            if let DieCommand::Point(_, _) = file.command {
                break;
            } else {
                file.next();
            }
        }

        for _ in 0..count {
            if let DieCommand::Point(is_outer, point) = file.command
                && is_outer == outer
            {
                points.push(ConvVertex { point, bulge: 0.0 });
                file.next();
            } else {
                match file.command {
                    DieCommand::NumOfArcs(_, _) => {
                        bail!("Insufficient number of vertices")
                    }
                    _ => break,
                }
            };
        }

        // skip excess
        while let DieCommand::Point(_, _) = file.command {
            file.next();
        }

        if let DieCommand::NumOfArcs(is_outer, count) = file.command
            && is_outer == outer
        {
            file.next();

            loop {
                if let DieCommand::Arc(_, _) = file.command {
                    break;
                } else {
                    file.next();
                }
            }

            for _ in 0..count {
                if let DieCommand::Arc(is_outer, arc) = file.command
                    && is_outer == outer
                {
                    arcs.push(arc);
                    file.next();
                } else {
                    break;
                };
            }

            // skip excess
            while let DieCommand::Arc(_, _) = file.command {
                file.next();
            }
        }

        if !points.is_empty() {
            if arcs.is_empty() {
                // No arcs
                conv_polyline(drawing, &points, line_type);
            } else if arcs.len() == 1
                && ((arcs[0].start == 0 && arcs[0].end == points.len() - 1)
                    || (arcs[0].end == 0 && arcs[0].start == points.len() - 1))
            {
                // Single arc
                let single_arc = arcs[0];
                conv_arc_by_points(
                    drawing,
                    single_arc.center,
                    points[single_arc.start].point,
                    points[single_arc.end].point,
                    single_arc.direction,
                    line_type,
                );
            } else {
                // Mixed
                arcs.sort_by_key(|a| a.start);
                let mut mixed = vec![];
                let mut cur_point = 0;
                let mut cur_arc = 0;
                while cur_point < points.len() && cur_arc < arcs.len() {
                    let mut pt = points[cur_point];
                    if cur_point == arcs[cur_arc].start {
                        let arc = arcs[cur_arc];
                        pt.bulge = conv_get_bulge(
                            arc.center,
                            points[arc.start].point,
                            points[arc.end].point,
                            arc.direction,
                        );
                        cur_point = arc.end;
                        cur_arc += 1;
                    } else {
                        cur_point += 1;
                    }
                    mixed.push(pt);
                }
                if cur_point < points.len() {
                    for pt in points.into_iter().skip(cur_point) {
                        mixed.push(pt);
                    }
                }
                conv_polyline(drawing, &mixed, line_type);
            }
        }
    }
    Ok(())
}

fn conv_parse_cuts(file: &mut DieFile, drawing: &mut Drawing) -> anyhow::Result<()> {
    let DieCommand::NumOfCuts(count) = file.command else {
        return Ok(());
    };
    file.next();

    // skip
    loop {
        if let DieCommand::NumOfPoints(_, _) = file.command {
            break;
        } else {
            file.next();
        }
    }

    for _ in 0..count {
        conv_parse_contour(file, drawing, false)?;
    }

    Ok(())
}

pub fn conv_load_die(filename: impl AsRef<Path>, drawing: &mut Drawing) -> anyhow::Result<()> {
    let mut file = DieFile::open(filename)?;

    file.next();
    loop {
        match file.command {
            DieCommand::EndOfFile => break,
            DieCommand::NumOfSign(_) => conv_parse_lines(&mut file, drawing),
            DieCommand::NumOfPoints(true, _) => conv_parse_contour(&mut file, drawing, true)?,
            DieCommand::NumOfCuts(_) => conv_parse_cuts(&mut file, drawing)?,
            _ => file.next(),
        }
    }

    Ok(())
}

pub fn convert_die(die_file: impl AsRef<Path>, dxf_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut drawing = dxf::Drawing::new();
    drawing.header.version = dxf::enums::AcadVersion::R2004;

    let line_type_main = dxf::tables::LineType {
        name: "CR_MAIN".into(),
        ..Default::default()
    };
    let line_type_thick = dxf::tables::LineType {
        name: "CR_THICK".into(),
        ..Default::default()
    };
    let line_type_thin = dxf::tables::LineType {
        name: "CR_THIN".into(),
        ..Default::default()
    };

    drawing.add_line_type(line_type_main);
    drawing.add_line_type(line_type_thick);
    drawing.add_line_type(line_type_thin);

    conv_load_die(die_file, &mut drawing)?;

    drawing.save_file(dxf_file)?;

    Ok(())
}
