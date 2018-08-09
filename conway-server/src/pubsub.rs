use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json;
use ws;

use conway::config::Settings;
use conway::{Game, GameConfig, Point, View};

pub fn listen(addr: &str) -> ws::Result<()> {
    ws::listen(addr, Server::new)
}

#[derive(Serialize)]
pub struct Message<'a> {
    status: Option<String>,
    pattern: Option<String>,
    #[serde(skip_serializing)]
    out: &'a ws::Sender,
}

impl<'a> Message<'a> {
    fn new(out: &'a ws::Sender) -> Self {
        Message {
            status: None,
            pattern: None,
            out,
        }
    }

    fn send(self) -> ws::Result<()> {
        self.out.send(self)
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

impl<'a> From<Message<'a>> for ws::Message {
    fn from(msg: Message) -> Self {
        ws::Message::Text(serde_json::to_string(&msg).unwrap())
    }
}

pub struct Server {
    out: ws::Sender,
    game: Arc<Mutex<Game>>,
    initial_game: Game,
    paused: bool,
}

impl Server {
    pub fn new(out: ws::Sender) -> Self {
        let game = Game::new(
            String::new().parse().unwrap(),
            Settings {
                delay: Duration::from_millis(100),
                view: View::Fixed,
                width: Some(50),
                height: Some(50),
                char_alive: 'x',
                char_dead: '.',
                ..Default::default()
            },
        );
        Server {
            out,
            game: Arc::new(Mutex::new(game.clone())),
            initial_game: game,
            paused: true,
        }
    }

    fn next_turn(&self, game: &mut Game) -> ws::Result<()> {
        if game.is_over() {
            self.message()
                .status("Game is stable.")
                .pattern(game.draw())
                .send()
        } else {
            game.tick();
            self.message().pattern(game.draw()).send()
        }
    }

    fn message(&self) -> Message {
        Message::new(&self.out)
    }
}

impl ws::Handler for Server {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        debug!("Received message: {:?}", msg);
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
                    Err(err) => return self.out.send(format!("WARNING: {}", err)),
                };
                game.scroll(dx, dy);
                self.message().pattern(game.draw()).send()
            }
            Some("center") => {
                game.center_viewport();
                self.message()
                    .status("Viewport centered on cell activity.")
                    .pattern(game.draw())
                    .send()
            }
            Some("new-grid") => {
                let data = args.next().unwrap_or_default();

                *game = match GameConfig::from_json(data).and_then(|config| config.build()) {
                    Ok(game) => game,
                    Err(err) => {
                        return self.message().status(err.to_string_chain()).send();
                    }
                };
                self.initial_game = game.clone();
                self.message()
                    .status("Started new game.")
                    .pattern(game.draw())
                    .send()
            }
            Some("restart") => {
                *game = self.initial_game.clone();
                self.message()
                    .status("Restarted game.")
                    .pattern(game.draw())
                    .send()
            }
            Some(arg) => self.out.send(format!(
                "WARNING: message contained unexpected command '{}'",
                arg
            )),
            None => self.out.send("WARNING: empty message received"),
        }
    }
}
