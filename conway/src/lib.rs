#![feature(transpose_result)]
#![recursion_limit = "1024"]

#[macro_use]
#[cfg(test)]
extern crate maplit;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

extern crate num_integer;

pub mod config;
pub mod game;
pub mod grid;
pub mod point;

// use std::error::Error;
// use std::fmt;
// use std::io;

pub use errors::*;

pub use config::Settings;
pub use game::{Game, View};
pub use grid::Grid;
pub use point::Point;

// pub type AppResult<T> = std::result::Result<T, AppError>;

// #[derive(Debug)]
// pub enum AppError {
//     IO(io::Error),
//     ParseInt(std::num::ParseIntError),
//     ParseChar(std::char::ParseCharError),
//     ParsePoint(String),
//     Msg(String),
//     WithCause(Box<AppError>, Box<Error + Send + Sync + 'static>),
// }

// impl AppError {
//     pub fn with_cause<E>(self, err: E) -> AppError
//     where
//         E: Error + Send + Sync + 'static,
//     {
//         AppError::WithCause(Box::new(self), Box::new(err))
//     }
// }

// impl fmt::Display for AppError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let (prefix, msg) = match self {
//             AppError::IO(e) => ("IO failed".to_owned(), e.to_string()),
//             AppError::ParseInt(e) => ("expected an integer".to_owned(), e.to_string()),
//             AppError::ParseChar(e) => ("expected a single character".to_owned(), e.to_string()),
//             AppError::ParsePoint(e) => ("failed to parse point".to_owned(), e.to_string()),
//             AppError::Msg(e) => ("invalid input".to_owned(), e.to_string()),
//             AppError::WithCause(e, cause) => (e.to_string(), cause.to_string()),
//         };
//         write!(f, "conway: {}: {}", prefix, msg)
//     }
// }

// impl Error for AppError {
//     fn cause(&self) -> Option<&Error> {
//         if let AppError::WithCause(_, ref err) = *self {
//             Some(&**err)
//         } else {
//             None
//         }
//     }
// }

// impl From<String> for AppError {
//     fn from(error: String) -> Self {
//         AppError::Msg(error)
//     }
// }

// impl From<io::Error> for AppError {
//     fn from(error: io::Error) -> Self {
//         AppError::IO(error)
//     }
// }

// impl From<std::num::ParseIntError> for AppError {
//     fn from(error: std::num::ParseIntError) -> Self {
//         AppError::ParseInt(error)
//     }
// }

// impl From<std::char::ParseCharError> for AppError {
//     fn from(error: std::char::ParseCharError) -> Self {
//         AppError::ParseChar(error)
//     }
// }

// We'll put our errors in an `errors` module, and other modules in
// this crate will `use errors::*;` to get access to everything
// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
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
