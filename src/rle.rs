// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Error, Position, Result};

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::digit1,
    combinator::{map, map_opt, opt},
    multi::many0,
    sequence::terminated,
    IResult,
};

pub struct RleFile {
    comments: Vec<Comment>,
    header: Header,
    pattern: Pattern,
}

pub struct Header {
    width: u32,
    height: u32,
}

pub struct RuleString {
    birth: Vec<u8>,
    survival: Vec<u8>,
}

pub enum CommentKind {
    Comment,
    Name,
    Author,
}

pub struct Comment {
    kind: CommentKind,
    text: String,
}

pub struct Pattern {
    runs: Vec<(u32, RunValue)>,
}

#[derive(Clone, Copy, Debug)]
enum RunValue {
    DeadCell,
    AliveCell,
    LineEnd,
}

fn parse_num(input: &str) -> IResult<&str, u32> {
    map_opt(digit1, |digits: &str| {
        if digits.starts_with("0") {
            None
        } else {
            digits.parse().ok()
        }
    })(input)
}

fn parse_dead_cell(input: &str) -> IResult<&str, RunValue> {
    map(tag("b"), |_| RunValue::DeadCell)(input)
}

fn parse_alive_cell(input: &str) -> IResult<&str, RunValue> {
    map(tag("o"), |_| RunValue::AliveCell)(input)
}

fn parse_line_end(input: &str) -> IResult<&str, RunValue> {
    map(tag("$"), |_| RunValue::LineEnd)(input)
}

fn parse_run(input: &str) -> IResult<&str, (u32, RunValue)> {
    let (input, len) = map(opt(parse_num), |x| x.unwrap_or(1))(input)?;
    let (input, value) = alt((parse_dead_cell, parse_alive_cell, parse_line_end))(input)?;
    Ok((input, (len, value)))
}

fn parse_whitespace(input: &str) -> IResult<&str, &str> {
    is_a(" \t\r\n")(input)
}

fn parse_rle(input: &str) -> IResult<&str, Pattern> {
    let (input, _) = opt(parse_whitespace)(input)?;
    let (input, runs) = many0(terminated(parse_run, opt(parse_whitespace)))(input)?;
    let (input, _) = tag("!")(input)?;
    Ok((input, Pattern { runs }))
}

impl Pattern {
    pub fn from_pattern(pattern: &str) -> Result<Self> {
        parse_rle(pattern)
            .map(|(_, rle)| rle)
            .map_err(|_| Error::RleParse)
    }

    pub(crate) fn alive_cells(&self) -> impl Iterator<Item = Position> + '_ {
        let (mut x, mut y): (i64, i64) = (0, 0);
        self.runs
            .iter()
            .filter_map(move |&(len, value)| {
                let len = i64::from(len);
                match value {
                    RunValue::AliveCell => {
                        x += len;
                        Some(((x - len)..x).map(move |i| Position::new(i, y)))
                    }
                    RunValue::DeadCell => {
                        x += len;
                        None
                    }
                    RunValue::LineEnd => {
                        x = 0;
                        y += len;
                        None
                    }
                }
            })
            .flat_map(|x| x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let glider = " b  ob  $ 2b\n\no\n$3o$$ $ $$$$ 5$ bbb $b$bb$bbb !  foobar";
        let rle = Pattern::from_pattern(glider).unwrap();
        let mut coords: Vec<_> = rle.alive_cells().collect();
        coords.sort();
        assert_eq!(
            coords,
            &[
                Position::new(0, 2),
                Position::new(1, 0),
                Position::new(1, 2),
                Position::new(2, 1),
                Position::new(2, 2)
            ]
        );
    }
}
