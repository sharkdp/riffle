use std::env;
use std::io;
use std::process;
use std::process::Command;
use std::str;

use riffle::Pager;

fn run() -> io::Result<()> {
    let mut args = env::args_os();
    args.next();

    let mut pager = Pager::new();

    let files = args.collect::<Vec<_>>();
    let output = Command::new("bat")
        .arg("--force-colorization")
        .args(files)
        .output()
        .expect("Failed to run 'bat'");

    let stdout = str::from_utf8(&output.stdout).expect("Could not decode 'bat' output");

    let lines: Vec<_> = stdout.lines().collect();

    // pager.on_redraw(|mut pager| {
    //     pager.append("1");
    // });
    pager.header(
        lines[0..3]
            .iter()
            .map(|l| format!("{}\n", l))
            .collect::<String>(),
    );

    for line in lines[3..].iter() {
        pager.append(&line);
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
