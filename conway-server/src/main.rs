extern crate conway;
extern crate conway_server;

use std::thread;

use conway_server::{http, pubsub};

const WEBSOCKET_ADDR: &str = "localhost:3012";

fn main() {
    thread::spawn(|| {
        pubsub::listen(WEBSOCKET_ADDR).unwrap();
    });
    http::rocket().launch();
}
