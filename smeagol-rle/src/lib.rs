//! Crate for working with run-length encoded (RLE) Life patterns.
#[macro_use]
extern crate nom;

use nom::{line_ending, not_line_ending};
use std::io::Read;

/// Matches any amount of whitespace.
named!(whitespace, take_while!(|c: u8| (c as char).is_whitespace()));

/// Matches a comment line in an RLE file, returning the comment without the leading `#` or
/// trailing newline.
named!(comment_line<&[u8], &[u8]>,
    do_parse!(
        char!('#') >>
        comment: not_line_ending >>
        line_ending >>
        (comment)
    )
);

/// Matches the header portion of an RLE file, returning the dimensions `(x, y)`.
///
/// TODO: also consume the Life rule.
named!(header<&[u8], (u32, u32)>,
    do_parse!(
        whitespace >>
        tag!("x") >>
        whitespace >>
        tag!("=") >>
        whitespace >>
        width: map_res!(nom::digit0, btoi::btoi) >>
        whitespace >>
        tag!(",") >>
        whitespace >>
        tag!("y") >>
        whitespace >>
        tag!("=") >>
        whitespace >>
        height: map_res!(nom::digit0, btoi::btoi) >>
        not_line_ending >>
        line_ending >>
        (width, height)
    )
);

/// Matches a non-negative number, returning the number or 1 if there are no digits.
fn parse_rle_digits(digits: &[u8]) -> Result<u32, btoi::ParseIntegerError> {
    if digits.len() == 0 {
        Ok(1)
    } else {
        btoi::btoi(digits)
    }
}

/// Matches a single unit in an RLE pattern.
named!(pattern_unit<&[u8], PatternUnit>, do_parse!(
    take_while!(|c: u8| (c as char).is_whitespace()) >>
    reps: map_res!(nom::digit0, parse_rle_digits) >>
    tag: one_of!("bo$") >>
    ( PatternUnit { reps, tag } )
));

/// Matches an entire RLE pattern string.
named!(pattern<&[u8], Vec<PatternUnit>>, do_parse!(
    units: many0!(pattern_unit) >>
    tag!("!") >>
    (units)
));

/// Matches an entire RLE file, returning the triple `(comments, (x, y), pattern_units)`.
named!(rle<&[u8], (Vec<&[u8]>, (u32, u32), Vec<PatternUnit>)>,
    do_parse!(
        comments: many0!(comment_line) >>
        dimensions: header >>
        units: many0!(pattern_unit) >>
        tag!("!") >>
        (comments, dimensions, units)
    )
);

/// An error than can occur.
#[derive(Debug)]
pub enum RleError {
    /// An IO error.
    Io(std::io::Error),
    /// A parsing error.
    Nom(nom::ErrorKind),
}

impl From<std::io::Error> for RleError {
    fn from(io_err: std::io::Error) -> Self {
        RleError::Io(io_err)
    }
}

impl From<nom::ErrorKind> for RleError {
    fn from(nom_err: nom::ErrorKind) -> Self {
        RleError::Nom(nom_err)
    }
}

/// A single unit in an RLE pattern.
///
/// A pattern unit consists of a character and a number indicating the repititions of that
/// character.
struct PatternUnit {
    reps: u32,
    tag: char,
}

/// A run-length encoded Life pattern.
pub struct Rle {
    units: Vec<PatternUnit>,
}

impl Rle {
    /// Loads an RLE pattern from the given file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// let rle = smeagol_rle::Rle::from_file("../assets/breeder1.rle")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file<P>(path: P) -> Result<Self, RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;

        let (_rest, (_comments, (_width, _height), units)) =
            rle(&buf).map_err(|e| e.into_error_kind())?;

        Ok(Self { units })
    }

    /// Reads an RLE pattern from the given byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// let rle = smeagol_rle::Rle::from_pattern(b"bob$2bo$3o!")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_pattern(pattern_str: &[u8]) -> Result<Self, RleError> {
        let (_rest, units) = pattern(pattern_str).map_err(|e| e.into_error_kind())?;
        Ok(Self { units })
    }

    /// Returns a `Vec` containing the coordinates of alive cells in the RLE pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// let rle = smeagol_rle::Rle::from_pattern(b"bob$2bo$3o!")?;
    ///
    /// for (x, y) in rle.alive_cells() {
    ///     // do something
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn alive_cells(&self) -> Vec<(u32, u32)> {
        let mut cells = vec![];
        // origin is at northwest corner
        let mut x = 0;
        let mut y = 0;
        for unit in &self.units {
            match unit.tag {
                // dead cells
                'b' => {
                    x += unit.reps;
                }
                // alive cells
                'o' => {
                    for _ in 0..unit.reps {
                        cells.push((x, y));
                        x += 1;
                    }
                }
                // ends of lines
                '$' => {
                    x = 0;
                    y += unit.reps;
                }
                _ => unreachable!(),
            }
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file() {
        Rle::from_file("../assets/breeder1.rle").unwrap();
    }

    #[test]
    #[should_panic]
    fn from_file_err() {
        Rle::from_file("nonexistent").unwrap();
    }

    #[test]
    fn from_pattern() {
        // integral sign
        Rle::from_pattern(b"3b2o$2bobo$2bo2b$obo2b$2o!").unwrap();
    }

    #[test]
    #[should_panic]
    fn from_pattern_err() {
        Rle::from_pattern(b"foo").unwrap();
    }

    #[test]
    fn alive_cells() {
        // glider
        let rle = Rle::from_pattern(b"bob$2bo$3o!").unwrap();
        let alive_cells = vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
        assert_eq!(rle.alive_cells(), alive_cells);
    }
}
