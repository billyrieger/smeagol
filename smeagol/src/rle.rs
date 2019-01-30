use std::io::Read;

use nom::{is_digit, line_ending, not_line_ending};

named!(comment_line<&[u8], String>,
    do_parse!(
        char!('#') >>
        comment: map!(not_line_ending, |x| std::str::from_utf8(x).expect("could not parse comment as utf8").to_owned()) >>
        line_ending >>
        (comment)
    )
);

named!(header<&[u8], (u32, u32)>,
    do_parse!(
        tag!("x") >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        tag!("=") >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        width: map!(take_while!(is_digit), |digits| std::str::from_utf8(digits).unwrap().parse::<u32>().unwrap()) >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        tag!(",") >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        tag!("y") >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        tag!("=") >>
        take_while!(|c: u8| (c as char).is_whitespace()) >>
        height: map!(take_while!(is_digit), |digits| std::str::from_utf8(digits).unwrap().parse::<u32>().unwrap()) >>
        not_line_ending >>
        line_ending >>
        (width, height)
    )
);

fn parse_rle_digits(digits: &[u8]) -> u32 {
    if digits.len() == 0 {
        1
    } else {
        std::str::from_utf8(digits).unwrap().parse::<u32>().unwrap()
    }
}

named!(pattern_unit<&[u8], (u32, char)>, do_parse!(
    take_while!(|c: u8| (c as char).is_whitespace()) >>
    reps: map!(take_while!(is_digit), parse_rle_digits) >>
    cell_type: one_of!("bo$") >>
    take_while!(|c: u8| (c as char).is_whitespace()) >>
    (reps, cell_type)
));

named!(pattern<&[u8], Vec<(u32, char)>>, many0!(pattern_unit));

named!(rle<&[u8], (Vec<String>, (u32, u32), Vec<(u32, char)>)>,
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
    pub width: u32,
    pub height: u32,
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

        let (_rest, (_comments, (width, height), units)) =
            rle(&buf).map_err(|e| e.into_error_kind())?;

        Ok(Self {
            width,
            height,
            units,
        })
    }

    pub fn from_pattern(width: u32, height: u32, pattern_str: &str) -> Result<Self, RleError> {
        let (_rest, units) =
            pattern(&pattern_str.bytes().collect::<Vec<_>>()).map_err(|e| e.into_error_kind())?;
        Ok(Self {
            width,
            height,
            units,
        })
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
