use std::env;
use std::io;
use std::process;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};

use riffle::{KeyCode, Pager};

struct Config {
    enable_wrapping: bool,
    show_sidebar: bool,
    run_editor: bool,
}

fn run() -> io::Result<()> {
    let mut args = env::args_os();
    args.next();
    let file = args.next().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "FILE argument missing",
    ))?;
    let file = &file;

    let config = Arc::new(Mutex::new(Config {
        enable_wrapping: true,
        show_sidebar: true,
        run_editor: false,
    }));

    let mut pager = Pager::new();

    let config2 = config.clone();
    pager.on_resize(move |pager| {
        let scroll_position = pager.scroll_position();
        pager.clear_buffer();

        let width = pager.terminal_width();

        let enable_wrapping = config2.lock().unwrap().enable_wrapping;
        let show_sidebar = config2.lock().unwrap().show_sidebar;

        let output = Command::new("bat")
            .arg(format!(
                "--style={}",
                if show_sidebar {
                    "full"
                } else {
                    "header,grid,snip"
                }
            ))
            .arg("--force-colorization")
            .arg("--paging=never")
            .arg(format!(
                "--wrap={}",
                if enable_wrapping {
                    "character"
                } else {
                    "never"
                }
            ))
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

        pager.scroll_to(scroll_position);
    });

    let config3 = config.clone();
    pager.on_keypress(move |pager, key| match key {
        KeyCode::Char('e') => {
            {
                config3.lock().unwrap().run_editor = true;
            }
            pager.quit();
        }
        KeyCode::Char('n') => {
            let show_sidebar = &mut config3.lock().unwrap().show_sidebar;
            *show_sidebar = !*show_sidebar;
        }
        KeyCode::Char('w') => {
            let enable_wrapping = &mut config3.lock().unwrap().enable_wrapping;
            *enable_wrapping = !*enable_wrapping;
        }
        _ => {}
    });

    pager.run();

    if config.lock().unwrap().run_editor {
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
