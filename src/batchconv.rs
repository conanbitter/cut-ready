use std::{
    ffi::OsStr,
    fs,
    path::{MAIN_SEPARATOR_STR, Path, PathBuf},
};

use anyhow::bail;
use walkdir::WalkDir;

use crate::diereader::convert_die;

const CLI_LOGO: &str = "  _____     __     ___              __    
 / ___/_ __/ /_   / _ \\___ ___ ____/ /_ __
/ /__/ // / __/  / , _/ -_) _ `/ _  / // /
\\___/\\_,_/\\__/  /_/|_|\\__/\\_,_/\\_,_/\\_, / 
                                   /___/
";

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

#[cfg(all(not(debug_assertions), windows))]
fn attach_console() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }

    println!("Console attached")
}

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

    /*println!("help {}", help);
    println!("convert {}", convert);
    println!("incl cubdirs {}", include_subdirs);
    println!("input {:?}", input);
    println!("output {:?}", output);*/

    if help {
        #[cfg(all(not(debug_assertions), windows))]
        attach_console();
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

    #[cfg(all(not(debug_assertions), windows))]
    attach_console();

    println!("{}", CLI_LOGO);

    if let Some(input) = input {
        if !input.exists() {
            println!("Input path \"{}\" desn't exist", input.display());
            return None;
        }

        let Some(abs_input) = std::path::absolute(input.clone()).ok() else {
            println!("Input path \"{}\" is wrong", input.display());
            return None;
        };

        let abs_output = if let Some(output) = output {
            let Some(res) = std::path::absolute(output.clone()).ok() else {
                println!("Output path \"{}\" is wrong", input.display());
                return None;
            };
            Some(res)
        } else {
            None
        };

        match convert_some(abs_input, abs_output, include_subdirs) {
            Ok(_) => {
                println!("Done!");
            }
            Err(err) => println!("ERROR: {}", err),
        }
    } else {
        println!("{}", CLI_USAGE);
    }
    None
}

fn convert_some(input: PathBuf, output: Option<PathBuf>, deep: bool) -> anyhow::Result<()> {
    //println!("Abs input: {}", input.display());
    //println!("Abs output: {:?}", output);

    if input.is_file() {
        //println!("Single file");
        let output = if let Some(output) = output {
            if is_dir(&output) {
                output.join(input.file_name().unwrap()).with_extension("die")
            } else {
                output
            }
        } else {
            input.with_extension("dxf")
        };
        convert_single(input, output)?;
    } else if input.is_dir() {
        if let Some(output) = output.as_ref()
            && !is_dir(output)
        {
            bail!("output must be a folder")
        }
        //println!("Multiple files");

        let files = find_files(&input, deep)?;
        for file in files {
            let output = match output.as_ref() {
                Some(outfolder) => outfolder.join(file.strip_prefix(&input)?).with_extension("dxf"),
                None => file.with_extension("dxf"),
            };
            convert_single(file, output)?;
        }
    } else {
        bail!("wrong input");
    }
    Ok(())
}

fn is_dir(path: &Path) -> bool {
    if path.exists() && path.is_dir() {
        true
    } else {
        let out_str = path.to_string_lossy();
        out_str.ends_with("/") || out_str.ends_with(MAIN_SEPARATOR_STR)
    }
}

fn find_files(path: &Path, deep: bool) -> anyhow::Result<Vec<PathBuf>> {
    let die_ext = Some(OsStr::new("die"));

    if deep {
        Ok(WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.path().extension() == die_ext)
            .map(|e| e.path().into())
            .collect())
    } else {
        let entries = fs::read_dir(path)?;

        Ok(entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension() == die_ext)
            .collect())
    }
}

fn ensure_folder_exist(file_path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn convert_single(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    println!("Converting:");
    println!(" ╭── {}", input.display());
    println!(" ╰─► {}\n", output.display());
    ensure_folder_exist(&output)?;
    convert_die(input, output)
}
