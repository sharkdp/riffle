use std::env;
use std::io;
use std::process;
use std::process::Command;
use std::str;

use riffle::Pager;

fn run() -> io::Result<()> {
    let mut pager = Pager::new();

    let mut args = env::args_os();
    args.next();
    let files = args.collect::<Vec<_>>();

    pager.on_resize(move |pager| {
        pager.clear_buffer();

        let width = pager.terminal_width();

        let output = Command::new("bat")
            .arg("--style=full")
            .arg("--force-colorization")
            .arg("--paging=never")
            .arg("--wrap=character")
            .arg(format!("--terminal-width={}", width))
            .args(&files)
            .output()
            .expect("Failed to run 'bat'");

        let stdout = str::from_utf8(&output.stdout).expect("Could not decode 'bat' output");
        let lines: Vec<_> = stdout.lines().collect();

        let len = lines.len();
        if len >= 4 {
            pager.header(
                lines[0..3]
                    .iter()
                    .map(|l| format!("{}\n", l))
                    .collect::<String>(),
            );

            for line in lines[3..(len - 1)].iter() {
                pager.append(&line);
            }

            pager.footer(lines[len - 1]);
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
