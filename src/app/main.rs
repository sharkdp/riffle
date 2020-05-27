use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process;

use flick::Flick;

fn run() -> io::Result<()> {
    let mut args = env::args_os();
    args.next();

    let stdin = io::stdin();

    let mut pager = Flick::new();

    let mut reader: Box<dyn BufRead> = if let Some(path) = args.next() {
        pager.footer(&path.to_string_lossy());
        let file = File::open(path)?;
        Box::new(BufReader::new(file))
    } else {
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
