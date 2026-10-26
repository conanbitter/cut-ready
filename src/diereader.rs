use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use tiny_skia::{PathBuilder, Rect};

use crate::{
    blueprint::{Blueprint, BlueprintLayer, LINEWIDTH_MAIN, LINEWIDTH_OTHER, LINEWIDTH_THICK},
    geometry::Point2D,
    pointerrors::PointErrors,
};

const DEFAULT_MAIN_COLOR: i32 = 0x007FFF;
const DEFAULT_CUTS_COLOR: i32 = 0x008181;
const DEFAULT_MARKS_COLOR: i32 = 0x000000;

#[derive(Debug)]
struct DieParam {
    int: i64,
    float: f64,
}

impl From<&str> for DieParam {
    fn from(value: &str) -> Self {
        let float = value.parse::<f64>().unwrap_or_default();
        let int = value.parse::<i64>().unwrap_or(float as i64);
        DieParam { int, float }
    }
}

#[derive(Debug, Clone, Copy)]
struct DieArc {
    start: usize,
    end: usize,
    center: Point2D,
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

fn parse_lines(file: &mut DieFile, pb: &mut PathBuilder) -> Rect {
    let DieCommand::NumOfSign(count) = file.command else {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0).unwrap();
    };

    let mut minx = f32::MAX;
    let mut maxx = f32::MIN;
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;
    for _ in 0..count {
        file.next();
        let DieCommand::Sign(p1, p2) = file.command else {
            break;
        };

        pb.move_to(p1.x, p1.y);
        pb.line_to(p2.x, p2.y);

        minx = minx.min(p1.x);
        maxx = maxx.max(p1.x);
        miny = miny.min(p1.y);
        maxy = maxy.max(p1.y);

        minx = minx.min(p2.x);
        maxx = maxx.max(p2.x);
        miny = miny.min(p2.y);
        maxy = maxy.max(p2.y);
    }

    Rect::from_ltrb(minx, miny, maxx, maxy).unwrap()
}

pub fn load_die<'a>(filename: impl AsRef<Path>) -> anyhow::Result<Blueprint<'a>> {
    let mut file = DieFile::open(filename)?;

    let mut main_pb = PathBuilder::new();
    let mut cuts_pb = PathBuilder::new();
    let mut marks_pb = PathBuilder::new();

    let mut bounds: Option<Rect> = None;

    loop {
        file.next();
        //println!("{:?}", file.command);

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
            _ => (),
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
