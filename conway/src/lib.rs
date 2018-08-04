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
extern crate num_integer;

#[macro_use]
mod macros;

pub mod config;
pub mod game;
pub mod grid;
pub mod point;

pub use config::Settings;
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
            ParseChar(::std::char::ParseCharError);
            ParseInt(::std::num::ParseIntError);
            IO(::std::io::Error);
        }
    }
}
