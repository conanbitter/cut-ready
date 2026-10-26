use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use rfd::MessageDialogResult::No;

pub fn load_die(filename: impl AsRef<Path>) -> anyhow::Result<()> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    loop {
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }
        let Some((command, params)) = line.split_once('=') else {
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

        println!("\"{}\"", command);
        for param in params {
            println!("    \"{}\"", param);
        }

        line.clear();
    }

    Ok(())
}
