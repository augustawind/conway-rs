use std::default::Default;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

use clap::ArgMatches;

use game::View;
use AppResult;

static SAMPLE_DIR: &str = "./sample_patterns";
static SAMPLE_CHOICES: &[&str] = &["beacon", "glider", "blinker", "toad"];
static VIEW_CHOICES: &[&str] = &["centered", "fixed", "follow"];

lazy_static! {
    static ref DEFAULT_CHAR_ALIVE: &'static str = "#";
    static ref DEFAULT_CHAR_DEAD: &'static str = "-";
    pub static ref CHAR_ALIVE: char = DEFAULT_CHAR_ALIVE.parse().unwrap();
    pub static ref CHAR_DEAD: char = DEFAULT_CHAR_DEAD.parse().unwrap();
}

fn parse_args<'a, I, T>(args: I) -> ArgMatches<'a>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    clap_app!(("Conway's Game of Life") =>
        (version: "0.1")
        (author: "Dustin Rohde <dustin.rohde@gmail.com>")
        (about: "A shell utility for running Conway's Game of Life simulations.")
        (@group source +required =>
            (@arg file: -F --file display_order(1)
                +takes_value
                "load a pattern from a file")
            (@arg sample: -S --sample display_order(1)
                default_value[glider]
                possible_values(SAMPLE_CHOICES)
                "load a sample pattern")
        )
        (@arg delay: -d --delay display_order(2)
            default_value("500")
            "delay (ms) between ticks")
        (@arg view: -v --view display_order(3)
            default_value[fixed]
            possible_values(VIEW_CHOICES)
            "viewing mode")
        (@arg width: -w --width display_order(4)
            +takes_value
            "viewport width [default: auto]")
        (@arg height: -h --height display_order(4)
            +takes_value
            "viewport height [default: auto]")
        (@arg live_char: -o --("live-char") display_order(5)
            default_value(*DEFAULT_CHAR_ALIVE)
            env[CONWAY_LIVE_CHAR]
            "character used to render live cells")
        (@arg dead_char: -x --("dead-char") display_order(5)
            default_value(*DEFAULT_CHAR_DEAD)
            env[CONWAY_DEAD_CHAR]
            "character used to render dead cells")
    ).get_matches_from(args)
}

#[derive(Debug)]
pub struct ConfigReader {
    pub settings: Settings,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub delay: Duration,
    pub view: View,

    pub width: Option<u64>,
    pub height: Option<u64>,

    pub char_alive: char,
    pub char_dead: char,
}

impl ConfigReader {
    pub fn from_env() -> AppResult<ConfigReader> {
        ConfigReader::from_args(env::args_os())
    }

    pub fn from_args<I, T>(args: I) -> AppResult<ConfigReader>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = parse_args(args);

        let conf = ConfigReader {
            settings: Settings {
                delay: Duration::from_millis(matches.value_of("delay").unwrap().parse()?),

                view: matches.value_of("view").unwrap().parse()?,

                width: matches.value_of("width").map(str::parse).transpose()?,
                height: matches.value_of("height").map(str::parse).transpose()?,

                char_alive: matches.value_of("live_char").unwrap().parse()?,
                char_dead: matches.value_of("dead_char").unwrap().parse()?,
            },
            pattern: {
                let path = if let Some(file) = matches.value_of("file") {
                    Path::new(file).to_path_buf()
                } else {
                    let file = matches.value_of("sample").unwrap();
                    Path::new(SAMPLE_DIR).join(file)
                };

                let mut f = File::open(path)?;
                let mut pattern = String::new();
                f.read_to_string(&mut pattern)?;
                pattern
            },
        };

        Ok(conf)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            delay: Duration::from_millis(500),
            view: View::Centered,
            width: Some(10),
            height: Some(10),
            char_alive: *CHAR_ALIVE,
            char_dead: *CHAR_DEAD,
        }
    }
}
