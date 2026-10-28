use std::path::PathBuf;

pub fn try_batch_convert() {
    let mut convert = false;
    let mut include_subdirs = false;
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-c" => convert = true,
            "-i" => include_subdirs = true,
            other => {
                if input.is_none() {
                    input = Some(other.into())
                } else if output.is_none() {
                    output = Some(other.into())
                }
            }
        }
    }

    println!("convert {}", convert);
    println!("incl cubdirs {}", include_subdirs);
    println!("input {:?}", input);
    println!("output {:?}", output);
}
