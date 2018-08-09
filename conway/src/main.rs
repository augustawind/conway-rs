extern crate conway;

use std::io;
use std::io::prelude::*;

use conway::{GameConfig, Result};

fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut io::stderr();
        e.write_err_chain(stderr);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut game = GameConfig::from_argv()?.build()?;
    let mut stdout = io::stdout();
    for frame in game.iter().with_delay(true) {
        write!(stdout, "\n{}", frame)?;
        stdout.flush()?;
    }
    Ok(())
}
