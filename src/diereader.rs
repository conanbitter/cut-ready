use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::geometry::Point2D;
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

#[derive(Debug)]
struct DieArc {
    start: u64,
    end: u64,
    direction: bool,
    center: Point2D,
}

#[derive(Debug)]
enum DieCommand {
    NumOfPoints(u64),
    Point(Point2D),
    NumOfCArcs(u64),
    CArc(DieArc),
    NumOfSign(u64),
    Sign(Point2D, Point2D),
    NumOfCuts(u64),
    NumOfCPoints(u64),
    CPoint(Point2D),
    NumOfCutArcs(u64),
    CutArc(DieArc),
    EndOfFile,
}

fn get_intparam(list: &[&str], index: usize) -> u64 {
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
    reader: BufReader<File>,
    line: String,
}

impl DieFile {
    fn open(filename: impl AsRef<Path>) -> anyhow::Result<DieFile> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        Ok(DieFile {
            reader,
            line: String::new(),
        })
    }

    fn next(&mut self) -> DieCommand {
        loop {
            let bytes_read = self.reader.read_line(&mut self.line).unwrap_or(0);

            if bytes_read == 0 {
                return DieCommand::EndOfFile;
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
                "num_of_points" => DieCommand::NumOfPoints(get_intparam(&params, 0)),
                "point" => DieCommand::Point(Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1))),
                "num_of_c_arcs" => DieCommand::NumOfCArcs(get_intparam(&params, 0)),
                "c_arc" => DieCommand::CArc(DieArc {
                    start: get_intparam(&params, 0),
                    end: get_intparam(&params, 1),
                    direction: get_intparam(&params, 2) > 0,
                    center: Point2D::new(get_floatparam(&params, 3), get_floatparam(&params, 4)),
                }),
                "num_of_sign" => DieCommand::NumOfSign(get_intparam(&params, 0)),
                "sign" => DieCommand::Sign(
                    Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1)),
                    Point2D::new(get_floatparam(&params, 2), get_floatparam(&params, 3)),
                ),
                "num_of_cuts" => DieCommand::NumOfCuts(get_intparam(&params, 0)),
                "num_of_cpoints" => DieCommand::NumOfCPoints(get_intparam(&params, 0)),
                "cpoint" => DieCommand::CPoint(Point2D::new(get_floatparam(&params, 0), get_floatparam(&params, 1))),
                "num_of_cut_arcs" => DieCommand::NumOfCutArcs(get_intparam(&params, 0)),
                "cut_arc" => DieCommand::CutArc(DieArc {
                    start: get_intparam(&params, 0),
                    end: get_intparam(&params, 1),
                    direction: get_intparam(&params, 2) > 0,
                    center: Point2D::new(get_floatparam(&params, 3), get_floatparam(&params, 4)),
                }),
                _ => {
                    self.line.clear();
                    continue;
                }
            };

            self.line.clear();
            return command;
        }
    }
}

pub fn load_die(filename: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut file = DieFile::open(filename)?;

    loop {
        let command = file.next();

        if let DieCommand::EndOfFile = command {
            break;
        }

        println!("{:?}", command);
    }

    Ok(())
}
