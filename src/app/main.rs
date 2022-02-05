use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process;

use riffle::Pager;

fn run() -> io::Result<()> {
    let mut args = env::args_os();
    args.next();

    let stdin = io::stdin();

    let mut pager = Pager::new();

    let mut reader: Box<dyn BufRead> = if let Some(path) = args.next() {
        pager.footer(format!("\x1b[7m{}\x1b[0m", path.to_string_lossy()));
        let file = File::open(path)?;
        Box::new(BufReader::new(file))
    } else {
        // Currently not supported, see https://github.com/crossterm-rs/crossterm/issues/396
        Box::new(stdin.lock())
    };

    let mut buffer = vec![];
    while let Ok(num) = reader.read_until(b'\n', &mut buffer) {
        if num == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&buffer);
        pager.append(&line);
        buffer.clear();
    }

    pager.run();

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
