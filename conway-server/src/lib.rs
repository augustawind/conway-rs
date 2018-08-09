#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate conway;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
extern crate ws;

pub mod http;
pub mod pubsub;
