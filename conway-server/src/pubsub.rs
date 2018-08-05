use std::fmt;
use std::io::{stderr, Write};
use std::sync::{Arc, Mutex};

use serde_json;
use ws;

use conway::{Game, Point, Settings, View};

pub fn listen(addr: &str) -> ws::Result<()> {
    ws::listen(addr, Server::new)
}

#[derive(Serialize)]
pub struct Message {
    status: Option<String>,
    pattern: Option<String>,
}

impl Message {
    fn new() -> Self {
        Message {
            status: None,
            pattern: None,
        }
    }

    fn status<T: ToString>(mut self, status: T) -> Self {
        self.status = Some(status.to_string());
        self
    }

    fn pattern<T: ToString>(mut self, pattern: T) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }
}

impl From<Message> for ws::Message {
    fn from(msg: Message) -> Self {
        ws::Message::Text(serde_json::to_string(&msg).unwrap())
    }
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
                view: View::Fixed,
                width: Some(50),
                height: Some(50),
                char_alive: 'x',
                char_dead: '.',
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
        self.out.send(Message::new().status(msg))
    }

    fn next_turn(&self, game: &mut Game) -> ws::Result<()> {
        if game.is_over() {
            self.out.send(
                Message::new()
                    .status("Pattern has stabilized. Start a new game.")
                    .pattern(game.draw()),
            )
        } else {
            game.tick();
            self.out.send(Message::new().pattern(game.draw()))
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
            Some("ping") => {
                if self.paused {
                    return Ok(());
                }
                self.next_turn(&mut game)
            }
            Some("step") => {
                if self.paused {
                    self.next_turn(&mut game)
                } else {
                    self.paused = true;
                    Ok(())
                }
            }
            Some("play") => {
                let was_paused = self.paused;
                self.paused = false;
                if was_paused {
                    return self.next_turn(&mut game);
                }
                Ok(())
            }
            Some("pause") => {
                self.paused = true;
                Ok(())
            }
            Some("scroll") => {
                let Point(dx, dy): Point = match args.next().unwrap_or_default().parse::<Point>() {
                    Ok(delta) => delta,
                    Err(err) => return self.alert(format!("WARNING: {}", err)),
                };
                game.scroll(dx, dy);
                self.out.send(Message::new().pattern(game.draw()))
            }
            Some("new-grid") => {
                let pattern = args.next().unwrap_or_default();
                game.reset_grid(pattern.parse().unwrap());
                self.out.send(Message::new().pattern(game.draw()))
            }
            Some(arg) => self.alert(format!(
                "WARNING: message contained unexpected command '{}'",
                arg
            )),
            None => self.alert("WARNING: empty message received"),
        }
    }
}
