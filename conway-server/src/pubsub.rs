use std::fmt;
use std::io::{stderr, Write};
use std::sync::{Arc, Mutex};

use ws;

use conway::{Game, Point, Settings, View};

// Grid defaults.
const CHAR_ALIVE: char = '■';
const CHAR_DEAD: char = '□';

// Commands.
const CMD_PING: &str = "ping";
const CMD_STEP: &str = "step";
const CMD_TOGGLE_PLAYBACK: &str = "toggle-playback";
const CMD_SCROLL: &str = "scroll";
const CMD_RESTART: &str = "restart";

pub fn listen(addr: &str) -> ws::Result<()> {
    ws::listen(addr, Server::new)
}

pub struct Server {
    out: ws::Sender,
    game: Arc<Mutex<Game>>,
    paused: bool,
}

impl Server {
    pub fn new(out: ws::Sender) -> Self {
        Server {
            out,
            game: Arc::new(Mutex::new(Server::new_game(String::new()))),
            paused: false,
        }
    }

    fn new_game(pattern: String) -> Game {
        Game::new(
            pattern.parse().unwrap(),
            Settings {
                char_alive: CHAR_ALIVE,
                char_dead: CHAR_DEAD,
                view: View::Fixed,
                width: Some(50),
                height: Some(50),
                ..Default::default()
            },
        )
    }

    fn set_game(&mut self, game: Game) {
        let mutex = Arc::get_mut(&mut self.game).unwrap();
        *mutex.get_mut().unwrap() = game;
    }

    fn alert<T: fmt::Display + Into<ws::Message>>(&self, msg: T) -> ws::Result<()> {
        write!(stderr(), "{}", msg)?;
        self.out.send(msg)
    }

    fn next_turn(&self, game: &mut Game) -> ws::Result<()> {
        if game.is_over() {
            self.out.send("Grid is empty. Start a new game.")
        } else {
            game.tick();
            self.out.send(game.draw())
        }
    }
}

impl ws::Handler for Server {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        self.set_game(Server::new_game(String::new()));
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let mut game: &mut Game = &mut self.game.lock().unwrap();

        let mut args = msg.as_text()?.trim().splitn(2, ' ');
        match args.next() {
            Some(cmd) if cmd == CMD_PING => {
                if self.paused {
                    return Ok(());
                }
                self.next_turn(&mut game)
            }
            Some(cmd) if cmd == CMD_STEP => {
                if !self.paused {
                    return Ok(());
                }
                self.next_turn(&mut game)
            }
            Some(cmd) if cmd == CMD_TOGGLE_PLAYBACK => {
                self.paused = !self.paused;
                if !self.paused {
                    return self.next_turn(&mut game);
                }
                Ok(())
            }
            Some(cmd) if cmd == CMD_SCROLL => {
                let Point(dx, dy): Point = match args.next().unwrap_or_default().parse::<Point>() {
                    Ok(delta) => delta,
                    Err(err) => return self.alert(format!("WARNING: {}", err)),
                };
                game.scroll(dx, dy);
                self.out.send(game.draw())
            }
            Some(cmd) if cmd == CMD_RESTART => {
                let pattern = args.next().unwrap_or_default();
                *game = Server::new_game(pattern.to_owned());
                Ok(())
            }
            Some(arg) => self.alert(format!(
                "WARNING: message contained unexpected command '{}'",
                arg
            )),
            None => self.alert("WARNING: empty message received"),
        }
    }
}
