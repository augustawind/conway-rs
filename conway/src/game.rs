use std::fmt;
use std::mem;
use std::str::FromStr;
use std::thread;

use num_integer::Integer;

use config::ConfigReader;
pub use config::Settings;
use grid::{Grid, Point};
use {Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Centered,
    Fixed,
    Follow,
}

impl FromStr for View {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "centered" => Ok(View::Centered),
            "fixed" => Ok(View::Fixed),
            "follow" => Ok(View::Follow),
            s => bail!("'{}' is not a valid choice", s),
        }
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                View::Centered => "centered",
                View::Fixed => "fixed",
                View::Follow => "follow",
            }
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Viewport {
    origin: Point,
    scroll: Point,
    width: u64,
    height: u64,
}

impl Viewport {
    pub fn new(width: u64, height: u64) -> Self {
        Viewport {
            width,
            height,
            ..Default::default()
        }
    }
}

pub struct GameIter<'a>(&'a mut Game);

impl<'a> Iterator for GameIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_over() {
            return None;
        }
        self.0.tick();
        thread::sleep(self.0.opts.delay);
        Some(self.0.draw())
    }
}

/// Game holds the high-level gameplay logic.
#[derive(Debug)]
pub struct Game {
    grid: Grid,
    swap: Grid,
    opts: Settings,
    viewport: Viewport,
}

impl Game {
    pub fn load() -> Result<Game> {
        let ConfigReader { settings, pattern } = ConfigReader::from_env()?;
        let grid = pattern.parse()?;
        Ok(Game::new(grid, settings))
    }

    pub fn new(grid: Grid, opts: Settings) -> Game {
        let mut swap = grid.clone();
        swap.clear();

        let (origin, Point(x1, y1)) = grid.calculate_bounds();
        let (width, height) = ((x1 - origin.0 + 1) as u64, (y1 - origin.1 + 1) as u64);

        let viewport = Viewport {
            origin,
            width: opts.width.unwrap_or(width),
            height: opts.height.unwrap_or(height),
            scroll: Point(0, 0),
        };

        Game {
            grid,
            swap,
            opts,
            viewport,
        }
    }

    pub fn iter(&mut self) -> GameIter {
        GameIter(self)
    }

    pub fn draw(&self) -> String {
        self.draw_viewport(self.viewport())
    }

    fn draw_viewport(&self, (Point(x0, y0), Point(x1, y1)): (Point, Point)) -> String {
        let mut output = String::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                output.push(if self.grid.is_alive(&Point(x, y)) {
                    self.opts.char_alive
                } else {
                    self.opts.char_dead
                });
            }
            output.push('\n');
        }
        output
    }

    pub fn scroll(&mut self, dx: i64, dy: i64) {
        self.viewport.scroll += Point(dx, dy);
    }

    pub fn center_viewport(&mut self) {
        let (origin, _) = self.viewport_centered();
        let Point(dx, dy) = origin - self.viewport.origin - self.viewport.scroll;
        self.scroll(dx, dy);
    }

    pub fn viewport(&self) -> (Point, Point) {
        match &self.opts.view {
            View::Fixed => self.viewport_fixed(),
            View::Centered => self.viewport_centered(),
            _ => unimplemented!(),
        }
    }

    // Return a Viewport starting at the lowest x and y values of live Cells present in the Game.
    pub fn viewport_fixed(&self) -> (Point, Point) {
        let Point(x0, y0) = self.viewport.origin + self.viewport.scroll;
        let p1 = Point(
            x0 + self.viewport.width as i64 - 1,
            y0 + self.viewport.height as i64 - 1,
        );
        (Point(x0, y0), p1)
    }

    // Return a Viewport centered around existing live Cells.
    pub fn viewport_centered(&self) -> (Point, Point) {
        let (Point(x0, y0), Point(x1, y1)) = self.grid.calculate_bounds();
        let (width, height) = (x1 - x0 + 1, y1 - y0 + 1);

        let (dx, dy) = (
            self.viewport.width as i64 - width,
            self.viewport.height as i64 - height,
        );

        let ((dx0, dx1), (dy0, dy1)) = (split_int(dx), split_int(dy));
        (
            Point(x0 - dx0, y0 - dy0) + self.viewport.scroll,
            Point(x1 + dx1, y1 + dy1) + self.viewport.scroll,
        )
    }

    /// Return whether the Game is over. This happens with the Grid is empty.
    pub fn is_over(&self) -> bool {
        self.grid.is_empty()
    }

    /// Execute the next turn in the Game of Life.
    ///
    /// `tick` applies the rules of game to each individual Point, killing some and reviving others.
    pub fn tick(&mut self) {
        for cell in self.grid.active_cells() {
            if self.survives(&cell) {
                self.swap.set_alive(cell);
            }
        }
        self.grid.clear();
        mem::swap(&mut self.grid, &mut self.swap);
    }

    /// Survives returns whether the cell at the given Point survives an application of The Rules.
    pub fn survives(&self, cell: &Point) -> bool {
        let live_neighbors = self.grid.live_neighbors(cell);
        if self.grid.is_alive(cell) {
            match live_neighbors {
                2 | 3 => true,
                _ => false,
            }
        } else {
            match live_neighbors {
                3 => true,
                _ => false,
            }
        }
    }
}

// Split an integer into 2 halves that always add up to the given number.
fn split_int<T: Integer + Copy>(n: T) -> (T, T) {
    let two = T::one() + T::one();
    let (quotient, remainder) = n.div_rem(&two);
    (quotient, quotient + remainder)
}

#[cfg(test)]
mod test {
    use super::*;

    fn mk_game(cells: Vec<Point>, (width, height): (Option<u64>, Option<u64>)) -> Game {
        Game::new(
            Grid::new(cells),
            Settings {
                width,
                height,
                ..Default::default()
            },
        )
    }

    // Viewport width/height should be taken from Settings if given.
    #[test]
    fn test_size_provided() {
        let game = mk_game(vec![Point(0, 0), Point(5, 5)], (Some(8), Some(8)));
        assert_eq!(game.viewport.width, 8);
        assert_eq!(game.viewport.height, 8);
    }

    // Viewport width/height should be derived from the Grid if not given in Settings.
    #[test]
    fn test_size_auto() {
        let game = mk_game(vec![Point(0, 0), Point(5, 5)], (None, None));
        assert_eq!(game.viewport.width, 6);
        assert_eq!(game.viewport.height, 6);
    }

    // Test `Game.survive`.
    #[test]
    fn test_survives() {
        let game = mk_game(vec![Point(1, 0), Point(1, 1), Point(1, 2)], (None, None));
        assert!(
            game.survives(&Point(1, 1)),
            "a live cell with 2 live neighbors should survive"
        );
        assert!(
            game.survives(&Point(0, 1)),
            "a dead cell with 3 live neighbors should survive"
        );
        assert!(
            game.survives(&Point(2, 1)),
            "a dead cell with 3 live neighbors should survive"
        );
        assert!(
            !game.survives(&Point(1, 0)),
            "a live cell with < 2 live neighbors should die"
        );
        assert!(
            !game.survives(&Point(1, 2)),
            "a live cell with < 2 live neighbors should die"
        );
    }

    // Tests for `Game.viewport` and related functionality.
    mod viewport {
        use super::*;

        // Test `Game.scroll`.
        #[test]
        fn test_scroll() {}

        // Test `Game.viewport_fixed`.
        #[test]
        fn test_viewport_fixed_1() {
            assert_eq!(
                mk_game(
                    vec![Point(2, 1), Point(-3, 0), Point(-2, 1), Point(-2, 0)],
                    (Some(7), Some(7)),
                ).viewport_fixed(),
                (Point(-3, 0), Point(3, 6)),
                "should pad content to fit width/height"
            );
        }

        // ...
        #[test]
        fn test_viewport_fixed_2() {
            assert_eq!(
                mk_game(
                    vec![Point(53, 4), Point(2, 1), Point(-12, 33)],
                    (Some(88), Some(12))
                ).viewport_fixed(),
                (Point(-12, 1), Point(75, 12))
            );
        }

        // ...
        #[test]
        fn test_viewport_fixed_3() {
            assert_eq!(
                mk_game(
                    vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                    (Some(10), Some(3))
                ).viewport_fixed(),
                (Point(2, 2), Point(11, 4)),
            );
        }

        // Test `Game.viewport_centered`.
        #[test]
        fn test_viewport_centered_1() {
            assert_eq!(
                mk_game(
                    vec![Point(2, 1), Point(-3, 0), Point(-2, 1), Point(-2, 0)],
                    (Some(7), Some(7)),
                ).viewport_centered(),
                (Point(-3, -2), Point(3, 4)),
                "should pad content to fit width/height"
            );
        }

        // ...
        #[test]
        fn test_viewport_centered_2() {
            assert_eq!(
                mk_game(
                    // natural size = 66 x 33
                    vec![Point(53, 4), Point(2, 1), Point(-12, 33)],
                    // adjust width: 88 - 66 = +22 / 2 => x0 - 11, x1 + 11
                    // adjust height: 12 - 33 = -21 / 2 => y0 + 10, y1 - 11
                    (Some(88), Some(12))
                ).viewport_centered(),
                // x0[-12] - 11 = -23 // x1[53] + 11 = 64
                // y0[1] + 10 = 11 // y1[33] - 11 = 22
                (Point(-23, 11), Point(64, 22))
            );
        }

        // ...
        #[test]
        fn test_viewport_centered_3() {
            assert_eq!(
                mk_game(
                    // natural size = 4 x 3
                    vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                    (Some(10), Some(3)),
                ).viewport_centered(),
                // x0[2] - 3 = -1 // x1[5] + 3 = 8
                // y0[2] + 0 = 2 // y1[4] + 0 = 4
                (Point(-1, 2), Point(8, 4)),
            );
        }

        // Test `Game.center_viewport`.
        #[test]
        fn test_center_viewport() {
            let mut game = mk_game(
                vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                (Some(10), Some(3)),
            );
            let expected = game.viewport_centered();
            game.center_viewport();
            assert_eq!(game.viewport_fixed(), expected);
        }

        // `Game.center_viewport` should account for current scroll.
        #[test]
        fn test_center_viewport_scrolled() {
            let mut game = mk_game(
                vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)],
                (Some(10), Some(3)),
            );
            game.scroll(-1, 2);
            let expected = game.viewport_centered();
            game.center_viewport();
            assert_eq!(game.viewport_fixed(), expected);
        }
    }

    // Test `split_int`.
    #[test]
    fn test_split_int() {
        assert_eq!(split_int(30), (15, 15));
        assert_eq!(split_int(31), (15, 16));
        assert_eq!(split_int(32), (16, 16));
        assert_eq!(split_int(0), (0, 0));
        assert_eq!(split_int(1), (0, 1));
        assert_eq!(split_int(2), (1, 1));
    }
}
