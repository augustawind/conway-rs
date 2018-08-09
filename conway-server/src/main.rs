#[macro_use]
extern crate log;
extern crate conway;
extern crate conway_server;
extern crate env_logger;

use std::thread;

use conway_server::{http, pubsub};

const WEBSOCKET_ADDR: &str = "localhost:3012";

fn main() {
    env_logger::init();
    thread::spawn(|| {
        pubsub::listen(WEBSOCKET_ADDR).unwrap();
    });
    let err = http::server().launch();
    error!("Error starting server: {:?}", err);
}
