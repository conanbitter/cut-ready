use std::{
    f32::consts::TAU,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use tiny_skia::{PathBuilder, Rect};

use crate::{
    blueprint::{Blueprint, BlueprintLayer, LINEWIDTH_MAIN, LINEWIDTH_OTHER, LINEWIDTH_THICK, arc_draw},
    geometry::{EPSILON, Point2D},
    pointerrors::PointErrors,
};

const DEFAULT_MAIN_COLOR: i32 = 0x007FFF;
const DEFAULT_CUTS_COLOR: i32 = 0x008181;
const DEFAULT_MARKS_COLOR: i32 = 0x000000;

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

pub fn new_layer<'a>(path: tiny_skia::Path, color: i32, width: f32) -> BlueprintLayer<'a> {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(r, g, b, 255);
    paint.anti_alias = true;

    BlueprintLayer {
        drawing: path,
        color: paint,
        line_width: width,
        order: 0,
    }
}

fn arc_draw_by_points(center: Point2D, start: Point2D, end: Point2D, direction: bool, pb: &mut PathBuilder) {
    let radius = (center.dist2(start).sqrt() + center.dist2(end).sqrt()) / 2.0;
    if start.dist2(end) < EPSILON {
        pb.push_circle(center.x, -center.y, radius);
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

fn parse_lines(file: &mut DieFile, pb: &mut PathBuilder) -> Rect {
    let DieCommand::NumOfSign(count) = file.command else {
        return Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap();
    };

    let mut minx = f32::MAX;
    let mut maxx = f32::MIN;
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;

    file.next();
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

    Rect::from_ltrb(minx, miny, maxx, maxy).unwrap()
}

fn parse_contour(file: &mut DieFile, pb: &mut PathBuilder, outer: bool) -> Rect {
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
                break;
            };
        }

        if let DieCommand::NumOfArcs(is_outer, count) = file.command
            && is_outer == outer
        {
            file.next();
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

        Rect::from_ltrb(minx, miny, maxx, maxy).unwrap()
    } else {
        Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap()
    }
}

fn parse_cuts(file: &mut DieFile, pb: &mut PathBuilder) -> Rect {
    let mut bounds: Option<Rect> = None;

    let DieCommand::NumOfCuts(count) = file.command else {
        return Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap();
    };
    file.next();

    for _ in 0..count {
        let new_bounds = parse_contour(file, pb, false);
        if let Some(bounds) = bounds.as_mut() {
            bounds.join(&new_bounds);
        } else {
            bounds = Some(new_bounds)
        }
    }

    bounds.unwrap_or(Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap())
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
                if let Some(total_bounds) = bounds.as_mut() {
                    total_bounds.join(&new_bounds);
                } else {
                    bounds = Some(new_bounds)
                }
            }
            DieCommand::NumOfPoints(true, _) => {
                let new_bounds = parse_contour(&mut file, &mut main_pb, true);
                if let Some(total_bounds) = bounds.as_mut() {
                    total_bounds.join(&new_bounds);
                } else {
                    bounds = Some(new_bounds)
                }
            }
            DieCommand::NumOfCuts(_) => {
                let new_bounds = parse_cuts(&mut file, &mut cuts_pb);
                if let Some(total_bounds) = bounds.as_mut() {
                    total_bounds.join(&new_bounds);
                } else {
                    bounds = Some(new_bounds)
                }
            }
            _ => file.next(),
        }
    }

    let mut layers = vec![];

    if let Some(path) = marks_pb.finish() {
        layers.push(new_layer(path, DEFAULT_MARKS_COLOR, LINEWIDTH_OTHER));
    }
    if let Some(path) = cuts_pb.finish() {
        layers.push(new_layer(path, DEFAULT_CUTS_COLOR, LINEWIDTH_THICK));
    }
    if let Some(path) = main_pb.finish() {
        layers.push(new_layer(path, DEFAULT_MAIN_COLOR, LINEWIDTH_MAIN));
    }

    Ok(Blueprint {
        layers,
        bounds: bounds.unwrap_or(Rect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap()),
        errors: PointErrors::default(),
    })
}
