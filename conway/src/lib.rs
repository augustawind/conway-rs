#![macro_use]
#![feature(transpose_result)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
extern crate num_integer;
extern crate serde_json;

pub mod config;
pub mod game;
pub mod grid;
pub mod point;

pub use config::GameConfig;
pub use errors::*;
pub use game::{Game, View};
pub use grid::Grid;
pub use point::Point;

mod errors {
    error_chain! {
        errors {
            ParsePoint(s: String) {
                description("failed to parse Point")
                display("failed to parse Point: {}", s)
            }
        }

        foreign_links {
            IO(::std::io::Error);
        }
    }
}
