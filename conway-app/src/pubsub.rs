use std::iter::FromIterator;
use std::ops::{Add, AddAssign};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json;
use ws;

use conway::config::Settings;
use conway::{Game, GameConfig, View};

pub fn listen(addr: &str) -> ws::Result<()> {
    ws::listen(addr, Server::new)
}

#[derive(Deserialize)]
pub enum Cmd {
    Ping,
    Step,
    Play,
    Pause,
    Scroll(i64, i64),
    Center,
    NewGrid(GameConfig),
    Restart,
}

#[derive(Debug, Serialize)]
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
        F: FnOnce(T) -> U,
    {
        match self {
            Message::Connected(t) => Message::Connected(f(t)),
            Message::Status(t) => Message::Status(f(t)),
            Message::Grid(t) => Message::Grid(f(t)),
            Message::Error(t) => Message::Error(f(t)),
        }
    }
}

impl<T: ToString + Serialize> From<Message<T>> for ws::Message {
    fn from(msg: Message<T>) -> Self {
        ws::Message::Text(serde_json::to_string(&msg).unwrap())
    }
}

impl<T: ToString> Add for Message<T> {
    type Output = MessageQueue;

    fn add(self, msg: Message<T>) -> MessageQueue {
        let mut queue = MessageQueue::new();
        queue.push(self);
        queue.push(msg);
        queue
    }
}

#[derive(Debug)]
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
                .map(|msg: Message<T>| msg.map(|s: T| s.to_string()))
                .collect(),
        )
    }

    fn drain<B>(&mut self) -> B
    where
        B: FromIterator<Message<String>>,
    {
        FromIterator::from_iter(self.0.drain(..))
    }

    fn flush(&mut self, out: &ws::Sender) -> ws::Result<()> {
        out.send(serde_json::to_string::<Vec<Message<String>>>(&self.drain()).unwrap())
    }
}

impl<'a> From<&'a MessageQueue> for ws::Message {
    fn from(&MessageQueue(ref msgs): &MessageQueue) -> Self {
        ws::Message::Text(serde_json::to_string(&msgs).unwrap())
    }
}

impl<T: ToString> Add<Message<T>> for MessageQueue {
    type Output = Self;
    fn add(mut self, msg: Message<T>) -> MessageQueue {
        self.push(msg);
        self
    }
}

impl<T: ToString> AddAssign<Message<T>> for MessageQueue {
    fn add_assign(&mut self, msg: Message<T>) {
        self.push(msg);
    }
}

struct State {
    game: Game,
    queue: MessageQueue,
}

pub struct Server {
    out: ws::Sender,
    state: Arc<Mutex<State>>,
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
                char_alive: 'x',
                char_dead: '.',
                ..Default::default()
            },
            (Some(50), Some(50)),
        );
        Server {
            out,
            state: Arc::new(Mutex::new(State {
                queue: MessageQueue::new(),
                game: game.clone(),
            })),
            initial_game: game,
            paused: true,
        }
    }

    fn next_turn(&self, game: &mut Game, queue: &mut MessageQueue) {
        if game.is_over() {
            queue.push(Message::Status("Grid has stabilized."));
        }
        game.tick();
        queue.push(Message::Grid(game.draw()));
    }
}

impl ws::Handler for Server {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            debug!("Connection with {} now open", addr);
        }
        let &mut State { ref mut queue, .. }: &mut State = &mut *self.state.lock().unwrap();
        queue.push(Message::Connected("Connected to game server."));
        queue.flush(&self.out)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        debug!("Received message: {:?}", msg);
        let &mut State {
            ref mut game,
            ref mut queue,
        }: &mut State = &mut *self.state.lock().unwrap();

        match serde_json::from_str(msg.as_text()?) {
            Ok(Cmd::Ping) => {
                if !self.paused {
                    self.next_turn(game, queue);
                }
            }
            Ok(Cmd::Step) => {
                if self.paused {
                    self.next_turn(game, queue);
                } else {
                    self.paused = true;
                }
            }
            Ok(Cmd::Play) => {
                if self.paused {
                    self.paused = false;
                    self.next_turn(game, queue);
                }
            }
            Ok(Cmd::Pause) => {
                self.paused = true;
            }
            Ok(Cmd::Scroll(dx, dy)) => {
                game.viewport.scroll(dx, dy);
                queue.push(Message::Grid(game.draw()));
            }
            Ok(Cmd::Center) => {
                game.center_viewport();
                queue.push(Message::Grid(game.draw()));
            }
            Ok(Cmd::NewGrid(config)) => {
                match config.build() {
                    Ok(new_game) => *game = new_game,
                    Err(err) => queue.push(Message::Error(err.to_string_chain())),
                };
                self.initial_game = game.clone();
                queue.push(Message::Status("Started a new game."));
                queue.push(Message::Grid(game.draw()));
            }
            Ok(Cmd::Restart) => {
                *game = self.initial_game.clone();
                queue.push(Message::Status("Restarted the current game."));
                queue.push(Message::Grid(game.draw()));
            }
            Err(err) => queue.push(Message::Error(format!("invalid input: {}", err))),
        };

        queue.flush(&self.out)
    }
}
