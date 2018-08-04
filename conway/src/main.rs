extern crate conway;

use std::io;
use std::io::prelude::*;

use conway::Game;

fn main() {
    let mut game = Game::load().unwrap();
    let mut stdout = io::stdout();
    for frame in game.iter() {
        write!(stdout, "\n{}", frame).unwrap();
        stdout.flush().unwrap();
    }
}
