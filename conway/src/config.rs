use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;

use clap::ArgMatches;

use game::View;
use {ErrorKind, Result};

const VIEW_CHOICES: &[&str] = &["centered", "fixed", "follow"];
const DEFAULT_CHAR_ALIVE: &str = "#";
const DEFAULT_CHAR_DEAD: &str = "-";

lazy_static! {
    static ref SAMPLE_DIR: PathBuf = PathBuf::from("./sample_patterns");
    pub static ref SAMPLE_PATTERNS: HashMap<&'static str, String> = hashmap!{
        "beacon" => string_from_file!(SAMPLE_DIR.join("beacon")),
        "blinker" => string_from_file!(SAMPLE_DIR.join("blinker")),
        "glider" => string_from_file!(SAMPLE_DIR.join("glider")),
        "toad" => string_from_file!(SAMPLE_DIR.join("toad")),
    };
    static ref SAMPLE_CHOICES: Vec<&'static str> = {
        let mut keys: Vec<&str> = SAMPLE_PATTERNS.keys().map(|k| *k).collect();
        keys.sort();
        keys
    };
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
                possible_values(SAMPLE_CHOICES.as_ref())
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
            default_value(DEFAULT_CHAR_ALIVE)
            env[CONWAY_LIVE_CHAR]
            "character used to render live cells")
        (@arg dead_char: -x --("dead-char") display_order(5)
            default_value(DEFAULT_CHAR_DEAD)
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
    pub fn from_env() -> Result<ConfigReader> {
        ConfigReader::from_args(env::args_os())
    }

    pub fn from_args<I, T>(args: I) -> Result<ConfigReader>
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
                if let Some(file) = matches.value_of("file") {
                    read_to_string(file)?
                } else {
                    let name = matches.value_of("sample").unwrap();
                    SAMPLE_PATTERNS
                        .get(name)
                        .ok_or_else(|| {
                            ErrorKind::Msg(format!("no sample pattern with name '{}'", name))
                        })?
                        .to_owned()
                }
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
