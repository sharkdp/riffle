use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process;
use std::str;

use riffle::Pager;

fn run() -> io::Result<()> {
    // Super simple pager application based on riffle

    // Unfortunately, reading from STDIN can not be supported currently.
    // See https://github.com/crossterm-rs/crossterm/issues/396

    let mut args = env::args_os();
    args.next();
    let input_path = args.next().expect("FILE argument");

    let mut pager = Pager::new();

    pager.on_init(|pager| {
        let input_file = File::open(&input_path).expect("Can not open file");
        let mut reader = BufReader::new(input_file);
        pager.footer(format!("\x1b[7m{}\x1b[0m", input_path.to_string_lossy()));

        let mut line_buffer = vec![];
        while let Ok(num) = reader.read_until(b'\n', &mut line_buffer) {
            if num == 0 {
                break;
            }

            pager.append(str::from_utf8(&line_buffer).unwrap());
            line_buffer.clear();
        }
    });

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
