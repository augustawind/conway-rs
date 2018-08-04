pub mod termion;

use num_integer::div_floor;

/// A Rect is a tuple struct containing the (x-origin, y-origin, width, height) of a rectangle.
#[derive(Debug)]
pub struct Rect {
    x0: u16,
    y0: u16,
    width: u16,
    height: u16,
}

impl Rect {
    /// Create a new Rect.
    pub fn new(x0: u16, y0: u16, width: u16, height: u16) -> Rect {
        Rect {
            x0: x0,
            y0: y0,
            width: width,
            height: height,
        }
    }

    /// Retrieve the Rect's origin X, origin Y, width and height.
    pub fn shape(&self) -> (u16, u16, u16, u16) {
        (self.x0, self.y0, self.width, self.height)
    }

    /// Retrieve the Rect's origin X and Y and it's opposite X and Y.
    pub fn coords(&self) -> (u16, u16, u16, u16) {
        let (x0, y0, width, height) = self.shape();
        (x0, y0, (x0 + width - 1), (y0 + height - 1))
    }

    pub fn resized(&self, dx: i16, dy: i16) -> Rect {
        let (x0, y0, x1, y1) = self.coords();
        let (dx, dy) = (div_floor(dx, 2), div_floor(dy, 2));
        let (x0, y0, x1, y1) = (
            x0 as i16 - dx,
            y0 as i16 - dy,
            x1 as i16 + dx,
            y1 as i16 + dy,
        );

        if x0 >= x1 || y0 >= y1 {
            panic!("cannot shrink Rect more than its own size");
        } else if x0 <= 0 || y0 <= 0 {
            panic!("cannot expand Rect out of bounds");
        }

        Rect {
            x0: x0 as u16,
            y0: y0 as u16,
            width: (x1 - x0 + 1) as u16,
            height: (y1 - y0 + 1) as u16,
        }
    }
}
