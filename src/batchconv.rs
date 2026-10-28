use std::path::PathBuf;

const CLI_LOGO: &str = "  _____     __     ___              __    
 / ___/_ __/ /_   / _ \\___ ___ ____/ /_ __
/ /__/ // / __/  / , _/ -_) _ `/ _  / // /
\\___/\\_,_/\\__/  /_/|_|\\__/\\_,_/\\_,_/\\_, / 
                                   /___/";

const CLI_USAGE: &str = "Usage:
  cut-ready <filename>
  cut-ready --convert [--include-sub] <input file/folder> [<output file/folder>]
  cut-ready --help

Options:
  -h --help         Show help screen

  -c --convert      Convert from DIE to DXF format a single file
                    or all files in a folder

  -i --include-sub  When converting all files in a folder,
                    also include files in al subfolders
    ";

// Returns:
// None - no further actions needed
// Some(none) - needs GUI window, no startup file
// Some(Some(_)) - needs GUI window, have startup file
pub fn try_batch_convert() -> Option<Option<PathBuf>> {
    let mut help = false;
    let mut convert = false;
    let mut include_subdirs = false;
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-c" | "--convert" => convert = true,
            "-i" | "--include-sub" => include_subdirs = true,
            "-h" | "--help" => help = true,
            "-ci" | "-ic" => {
                convert = true;
                include_subdirs = true;
            }
            other => {
                if input.is_none() {
                    input = Some(other.into())
                } else if output.is_none() {
                    output = Some(other.into())
                }
            }
        }
    }

    println!("help {}", help);
    println!("convert {}", convert);
    println!("incl cubdirs {}", include_subdirs);
    println!("input {:?}", input);
    println!("output {:?}", output);

    if help {
        println!("{}", CLI_LOGO);
        println!("{}", CLI_USAGE);
        return None;
    }

    if !convert {
        if let Some(infile) = input
            && infile.is_file()
        {
            return Some(Some(infile));
        } else {
            return Some(None);
        }
    }

    if let Some(input) = input {
    } else {
    }
    None
}
