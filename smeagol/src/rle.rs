use std::io::Read;

use nom::{line_ending, not_line_ending};

named!(whitespace, take_while!(|c: u8| (c as char).is_whitespace()));

named!(comment_line<&[u8], &[u8]>,
    do_parse!(
        char!('#') >>
        comment: not_line_ending >>
        line_ending >>
        (comment)
    )
);

named!(header<&[u8], (u32, u32)>,
    do_parse!(
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

fn parse_rle_digits(digits: &[u8]) -> Result<u32, btoi::ParseIntegerError> {
    if digits.len() == 0 {
        Ok(1)
    } else {
        btoi::btoi(digits)
    }
}

named!(pattern_unit<&[u8], (u32, char)>, do_parse!(
    take_while!(|c: u8| (c as char).is_whitespace()) >>
    reps: map_res!(nom::digit0, parse_rle_digits) >>
    cell_type: one_of!("bo$") >>
    (reps, cell_type)
));

named!(pattern<&[u8], Vec<(u32, char)>>, many0!(pattern_unit));

named!(rle<&[u8], (Vec<&[u8]>, (u32, u32), Vec<(u32, char)>)>,
    do_parse!(
        comments: many0!(comment_line) >>
        dimensions: header >>
        units: many0!(pattern_unit) >>
        tag!("!") >>
        (comments, dimensions, units)
    )
);

#[derive(Debug)]
pub enum RleError {
    Io(std::io::Error),
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

pub struct Rle {
    units: Vec<(u32, char)>,
}

impl Rle {
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

    pub fn from_pattern(pattern_str: &[u8]) -> Result<Self, RleError> {
        let (_rest, units) = pattern(pattern_str).map_err(|e| e.into_error_kind())?;
        Ok(Self { units })
    }

    pub fn alive_cells(&self) -> Vec<(u32, u32)> {
        let mut cells = vec![];
        // origin is at northwest corner
        let mut x = 0;
        let mut y = 0;
        for &(reps, character) in &self.units {
            match (reps, character) {
                // dead cells
                (n, 'b') => {
                    x += n;
                }
                // alive cells
                (n, 'o') => {
                    for _ in 0..n {
                        cells.push((x, y));
                        x += 1;
                    }
                }
                // end of lines
                (n, '$') => {
                    x = 0;
                    y += n;
                }
                _ => unreachable!(),
            }
        }
        cells
    }
}
