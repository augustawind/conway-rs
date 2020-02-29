#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate conway;
#[macro_use]
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate ws;

pub mod http;
pub mod pubsub;
