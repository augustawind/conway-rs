use std::collections::BTreeMap;
use std::default::Default;
use std::env;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::Path;
use std::time::Duration;

use clap::ArgMatches;
use serde_json;

use game::{Game, View};
use {ErrorKind, Result, ResultExt};

const VIEW_CHOICES: &[&str] = &["centered", "fixed", "follow"];
const DEFAULT_CHAR_ALIVE: &str = "#";
const DEFAULT_CHAR_DEAD: &str = "-";

lazy_static! {
    static ref SAMPLE_DIR: &'static Path = Path::new("./sample_patterns");
    pub static ref SAMPLE_PATTERNS: BTreeMap<&'static str, &'static str> = btreemap! {
        "beacon" => include_str!("../sample_patterns/blinker"),
        "blinker" => include_str!("../sample_patterns/blinker"),
        "glider" => include_str!("../sample_patterns/glider"),
        "toad" => include_str!("../sample_patterns/toad"),
    };
    static ref SAMPLE_CHOICES: Vec<&'static str> = SAMPLE_PATTERNS.keys().copied().collect();
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
                +takes_value
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
            default_value[auto]
            "viewport width")
        (@arg height: -h --height display_order(4)
            default_value[auto]
            "viewport height")
        (@arg live_char: -o --("live-char") display_order(5)
            default_value(DEFAULT_CHAR_ALIVE)
            env[CONWAY_LIVE_CHAR]
            "character used to render live cells")
        (@arg dead_char: -x --("dead-char") display_order(5)
            default_value(DEFAULT_CHAR_DEAD)
            env[CONWAY_DEAD_CHAR]
            "character used to render dead cells")
    )
    .get_matches_from(args)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub delay: Duration,
    pub view: View,
    pub char_alive: char,
    pub char_dead: char,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            delay: Duration::from_millis(500),
            view: View::Centered,
            char_alive: *CHAR_ALIVE,
            char_dead: *CHAR_DEAD,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameConfig {
    #[serde(default)]
    pub settings: Settings,
    pub pattern: String,
    pub bounds: (Option<u64>, Option<u64>),
}

impl GameConfig {
    pub fn build(self) -> Result<Game> {
        Ok(Game::new(self.pattern.parse()?, self.settings, self.bounds))
    }

    pub fn from_json(s: &str) -> Result<Self> {
        serde_json::from_str(s).chain_err(|| "failed to read config from json")
    }

    pub fn from_argv() -> Result<Self> {
        GameConfig::from_args(env::args_os())
    }

    pub fn from_args<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = parse_args(args);

        let conf = GameConfig {
            settings: Settings {
                delay: Duration::from_millis(
                    matches
                        .value_of("delay")
                        .unwrap()
                        .parse()
                        .map_err(|_| ErrorKind::ParseArg("delay", "an integer"))?,
                ),

                view: matches.value_of("view").unwrap().parse()?,

                char_alive: matches
                    .value_of("live_char")
                    .unwrap()
                    .parse()
                    .map_err(|_| ErrorKind::ParseArg("live_char", "a character"))?,
                char_dead: matches
                    .value_of("dead_char")
                    .unwrap()
                    .parse()
                    .map_err(|_| ErrorKind::ParseArg("dead_char", "a character"))?,
            },
            pattern: {
                if let Some(file) = matches.value_of("file") {
                    read_to_string(file)?
                } else {
                    SAMPLE_PATTERNS
                        .get(matches.value_of("sample").unwrap())
                        .expect("unknown sample pattern")
                        .to_string()
                }
            },
            bounds: (
                matches
                    .value_of("width")
                    .and_then(|s| if s == "auto" { None } else { Some(s.parse()) })
                    .transpose()
                    .map_err(|_| ErrorKind::ParseArg("width", "an integer"))?,
                matches
                    .value_of("height")
                    .and_then(|s| if s == "auto" { None } else { Some(s.parse()) })
                    .transpose()
                    .map_err(|_| ErrorKind::ParseArg("height", "an integer"))?,
            ),
        };

        Ok(conf)
    }
}
