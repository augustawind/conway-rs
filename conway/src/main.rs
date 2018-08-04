extern crate conway;

use std::io;
use std::io::prelude::*;

use conway::{Game, Result};

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut game = Game::load()?;
    let mut stdout = io::stdout();
    for frame in game.iter() {
        write!(stdout, "\n{}", frame)?;
        stdout.flush()?;
    }
    Ok(())
}
