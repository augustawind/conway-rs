use std::mem;
use std::str::FromStr;
use std::thread;

use num_integer::Integer;

use config::ConfigReader;
pub use config::Settings;
use grid::{Grid, Point};
use {AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Centered,
    Fixed,
    Follow,
}

impl FromStr for View {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "centered" => Ok(View::Centered),
            "fixed" => Ok(View::Fixed),
            "follow" => Ok(View::Follow),
            s => Err(From::from(format!("'{}' is not a valid choice", s))),
        }
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
    pub fn load() -> AppResult<Game> {
        let ConfigReader { settings, pattern } = ConfigReader::from_env()?;
        let grid = pattern.parse()?;
        Ok(Game::new(grid, settings))
    }

    pub fn new(grid: Grid, opts: Settings) -> Game {
        let mut swap = grid.clone();
        swap.clear();

        let (origin, Point(x1, y1)) = grid.calculate_bounds();
        let (width, height) = ((x1 - origin.0 + 1) as u64, (y1 - origin.1 + 1) as u64);

        // FIXME: implement Option instead of relying on 0
        // set min dimensions to at least the starting Grid's natural size
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
        self.viewport.scroll = self.viewport.scroll - Point(dx, dy);
    }

    pub fn viewport(&self) -> (Point, Point) {
        match &self.opts.view {
            View::Fixed => self.viewport_fixed(),
            View::Centered => self.viewport_centered(),
            _ => unimplemented!(),
        }
    }

    pub fn viewport_fixed(&self) -> (Point, Point) {
        let Point(x0, y0) = self.viewport.origin + self.viewport.scroll;
        let p1 = Point(
            x0 + self.viewport.width as i64,
            y0 + self.viewport.height as i64,
        );
        (Point(x0, y0), p1)
    }

    pub fn viewport_centered(&self) -> (Point, Point) {
        let (Point(x0, y0), Point(x1, y1)) = self.grid.calculate_bounds();
        let (width, height) = (x1 - x0 + 1, y1 - y0 + 1);

        let (dx, dy) = (
            self.viewport.width as i64 - width,
            self.viewport.height as i64 - height,
        );

        let ((dx0, dx1), (dy0, dy1)) = (split_int(dx), split_int(dy));
        (Point(x0 - dx0, y0 - dy0), Point(x1 + dx1, y1 + dy1))
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

fn split_int<T: Integer + Copy>(n: T) -> (T, T) {
    let two = T::one() + T::one();
    let (quotient, remainder) = n.div_rem(&two);
    (quotient, quotient + remainder)
}

#[cfg(test)]
mod test {
    use super::*;

    // FIXME: implement Option for width/height to achieve this
    // #[test]
    // fn test_min_size() {
    //     let game = Game::new(
    //         Grid::new(vec![Point(0, 0), Point(5, 5)]),
    //         Settings {
    //             min_width: 8,
    //             min_height: 8,
    //             ..Default::default()
    //         },
    //     );
    //     assert_eq!((game.opts.min_width, game.opts.min_height), (8, 8),);
    // }

    // #[test]
    // fn test_min_size_override() {
    //     let game = Game::new(
    //         Grid::new(vec![Point(0, 0), Point(5, 5)]),
    //         Settings {
    //             min_width: 3,
    //             min_height: 3,
    //             ..Default::default()
    //         },
    //     );
    //     assert_eq!(
    //         (game.opts.min_width, game.opts.min_height),
    //         (6, 6),
    //         "natural size should override given min size if natural > given"
    //     );
    // }

    #[test]
    fn test_survives_blinker() {
        let game = Game::new(
            Grid::new(vec![Point(1, 0), Point(1, 1), Point(1, 2)]),
            Default::default(),
        );
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

    mod viewport {
        use super::*;

        #[test]
        fn test_viewport_centered_1() {
            assert_eq!(
                Game::new(
                    Grid::new(vec![Point(2, 1), Point(-3, 0), Point(-2, 1), Point(-2, 0)]),
                    Settings {
                        width: Some(7),
                        height: Some(7),
                        ..Default::default()
                    }
                ).viewport_centered(),
                (Point(-3, -2), Point(3, 4)),
                "should pad content to fit width/height"
            );
        }

        #[test]
        fn test_viewport_centered_2() {
            assert_eq!(
                Game::new(
                    // natural size = 66 x 33
                    Grid::new(vec![Point(53, 4), Point(2, 1), Point(-12, 33)]),
                    Settings {
                        // adjust width: 88 - 66 = +22 / 2 => x0 - 11, x1 + 11
                        width: Some(88),
                        // adjust height: 12 - 33 = -21 / 2 => y0 + 10, y1 - 11
                        height: Some(12),
                        ..Default::default()
                    }
                ).viewport_centered(),
                // x0[-12] - 11 = -23 // x1[53] + 11 = 64
                // y0[1] + 10 = 11 // y1[33] - 11 = 22
                (Point(-23, 11), Point(64, 22))
            );
        }

        #[test]
        fn test_viewport_centered_3() {
            assert_eq!(
                Game::new(
                    // natural size = 4 x 3
                    Grid::new(vec![Point(2, 3), Point(3, 3), Point(5, 4), Point(4, 2)]),
                    Settings {
                        // adjust width: 10 - 4 = +6 / 2 => x0 - 3, x1 + 3
                        width: Some(10),
                        // adjust height: 3 - 3 = 0 => N/A
                        height: Some(3),
                        ..Default::default()
                    }
                ).viewport_centered(),
                // x0[2] - 3 = -1 // x1[5] + 3 = 8
                // y0[2] + 0 = 2 // y1[4] + 0 = 4
                (Point(-1, 2), Point(8, 4)),
            );
        }
    }

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
