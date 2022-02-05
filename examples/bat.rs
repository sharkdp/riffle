use std::env;
use std::io;
use std::process;
use std::process::Command;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use riffle::{KeyCode, Pager};

fn run() -> io::Result<()> {
    let mut args = env::args_os();
    args.next();
    let file = args.next().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "FILE argument missing",
    ))?;
    let file = &file;

    let mut pager = Pager::new();
    pager.on_resize(move |pager| {
        pager.clear_buffer();

        let width = pager.terminal_width();

        let output = Command::new("bat")
            .arg("--style=full")
            .arg("--force-colorization")
            .arg("--paging=never")
            .arg("--wrap=character")
            .arg(format!("--terminal-width={}", width))
            .arg(file)
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

    let open_editor = Arc::new(AtomicBool::new(false));
    let open_editor_c = open_editor.clone();
    pager.on_keypress(move |pager, key| match key {
        KeyCode::Char('e') => {
            open_editor_c.store(true, Ordering::Relaxed);
            pager.quit();
        }
        _ => {}
    });

    pager.run();

    if open_editor.load(Ordering::Relaxed) {
        Command::new(std::env::var_os("EDITOR").expect("EDITOR not set"))
            .arg(file)
            .status()
            .expect("Failed to run editor");
    }

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
