use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json;
use ws;

use conway::config::Settings;
use conway::{Game, GameConfig, Point, View};

pub fn listen(addr: &str) -> ws::Result<()> {
    ws::listen(addr, Server::new)
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "content")]
pub enum Message<T> {
    Connected(T),
    Status(T),
    Grid(T),
    Error(T),
}

impl<T> Message<T> {
    fn map<U, F>(self, f: F) -> Message<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self {
            Message::Connected(t) => Message::Connected(f(&t)),
            Message::Status(t) => Message::Status(f(&t)),
            Message::Grid(t) => Message::Grid(f(&t)),
            Message::Error(t) => Message::Error(f(&t)),
        }
    }
}

impl<T: ToString + Serialize> Message<T> {
    fn send(self, out: &ws::Sender) -> ws::Result<()> {
        out.send(self)
    }
}

impl<T: ToString + Serialize> From<Message<T>> for ws::Message {
    fn from(msg: Message<T>) -> Self {
        ws::Message::Text(serde_json::to_string(&msg).unwrap())
    }
}

pub struct MessageQueue(Vec<Message<String>>);

impl MessageQueue {
    fn new() -> Self {
        MessageQueue(Vec::new())
    }

    fn push<T: ToString>(&mut self, msg: Message<T>) {
        self.0.push(msg.map(|s| s.to_string()));
    }

    fn append<T: ToString>(&mut self, msgs: Vec<Message<T>>) {
        self.0.append(
            &mut msgs
                .into_iter()
                .map(|msg: Message<T>| msg.map(|s: &T| s.to_string()))
                .collect(),
        )
    }

    fn flush<B>(&mut self) -> B
    where
        B: FromIterator<Message<String>>,
    {
        FromIterator::from_iter(self.0.drain(..))
    }
}

impl From<MessageQueue> for ws::Message {
    fn from(mut queue: MessageQueue) -> Self {
        ws::Message::Text(serde_json::to_string::<Vec<Message<String>>>(&queue.flush()).unwrap())
    }
}

pub struct Server {
    out: ws::Sender,
    queue: MessageQueue,
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
            queue: MessageQueue::new(),
            game: Arc::new(Mutex::new(game.clone())),
            initial_game: game,
            paused: true,
        }
    }

    fn next_turn(&self, game: &mut Game) -> ws::Result<()> {
        if game.is_over() {
            Message::Status("Grid has stabilized.").send(&self.out)
        } else {
            game.tick();
            Message::Grid(game.draw()).send(&self.out)
        }
    }
}

impl ws::Handler for Server {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            debug!("Connection with {} now open", addr);
        }
        Message::Connected("Connected to game server.").send(&self.out)
    }

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
                    Err(err) => return Message::Error(err.to_string()).send(&self.out),
                };
                game.scroll(dx, dy);
                Message::Grid(game.draw()).send(&self.out)
            }
            Some("center") => {
                game.center_viewport();
                Message::Grid(game.draw()).send(&self.out)
            }
            Some("new-grid") => {
                let data = args.next().unwrap_or_default();

                *game = match GameConfig::from_json(data).and_then(|config| config.build()) {
                    Ok(game) => game,
                    Err(err) => return Message::Error(err.to_string_chain()).send(&self.out),
                };
                self.initial_game = game.clone();
                Message::Grid(game.draw()).send(&self.out)
            }
            Some("restart") => {
                *game = self.initial_game.clone();
                Message::Grid(game.draw()).send(&self.out)
            }
            Some(arg) => {
                Message::Error(format!("received unexpected command '{}'", arg)).send(&self.out)
            }
            None => Message::Error("received empty message").send(&self.out),
        }
    }
}
