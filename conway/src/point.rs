use std::fmt;
use std::num::ParseIntError;
use std::ops;
use std::str::FromStr;

use {Error, ErrorKind, Result};

/// A Point is a point on the `Grid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point(pub i64, pub i64);

impl ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Point) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Default for Point {
    fn default() -> Self {
        Point(0, 0)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Point(x, y) = self;
        write!(f, "({}, {})", x, y)
    }
}

impl FromStr for Point {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (lparen, rest) = s.trim().split_at(1);
        if lparen != "(" {
            bail!(ErrorKind::ParsePoint(format!(
                "unexpected character '{}'",
                lparen
            )));
        }
        let (rest, rparen) = rest.split_at(rest.len() - 1);
        if rparen != ")" {
            bail!(ErrorKind::ParsePoint(format!(
                "unexpected character '{}'",
                rparen
            )));
        }
        let mut nums = rest.split(',');
        let x: i64 = nums
            .next()
            .ok_or_else(|| ErrorKind::ParsePoint(format!("missing value for x")))?
            .trim()
            .parse()
            .map_err(|e: ParseIntError| ErrorKind::ParsePoint(e.to_string()))?;
        let y: i64 = nums
            .next()
            .ok_or_else(|| ErrorKind::ParsePoint(format!("missing value for y")))?
            .trim()
            .parse()
            .map_err(|e: ParseIntError| ErrorKind::ParsePoint(e.to_string()))?;
        Ok(Point(x, y))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Point(1, 1) + Point(4, 9), Point(5, 10));
        assert_eq!(Point(-3, 5) + Point(-5, -6), Point(-8, -1));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Point(1, 1) - Point(4, 9), Point(-3, -8));
        assert_eq!(Point(-3, 5) - Point(-5, -6), Point(2, 11));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("(-4, 9)".parse::<Point>().unwrap(), Point(-4, 9));
    }
}
