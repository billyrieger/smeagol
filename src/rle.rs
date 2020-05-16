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

pub struct Rle {
    runs: Vec<Run>,
}

impl Rle {
    pub fn from_pattern(pattern: &str) -> Result<Self> {
        parse_rle(pattern.as_bytes())
            .map(|(_, rle)| rle)
            .map_err(|_| Error::RleParse)
    }

    pub fn alive_cells(&self) -> impl Iterator<Item = Position> + '_ {
        let (mut x, mut y): (i64, i64) = (0, 0);
        self.runs
            .iter()
            .flat_map(move |run| {
                let len = i64::from(run.len);
                match run.value {
                    RunValue::DeadCell => {
                        x += len;
                        None
                    }
                    RunValue::AliveCell => {
                        x += len;
                        Some(((x - len)..x).map(move |i| Position::new(i, y)))
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

#[derive(Clone, Copy, Debug)]
struct Run {
    len: u32,
    value: RunValue,
}

impl Run {
    fn new(len: u32, value: RunValue) -> Self {
        Self { len, value }
    }
}

#[derive(Clone, Copy, Debug)]
enum RunValue {
    DeadCell,
    AliveCell,
    LineEnd,
}

fn parse_num(input: &[u8]) -> IResult<&[u8], u32> {
    map_opt(digit1, |digits: &[u8]| {
        if digits[0] == b'0' {
            None
        } else {
            String::from_utf8_lossy(digits).parse().ok()
        }
    })(input)
}

fn dead_cell(input: &[u8]) -> IResult<&[u8], RunValue> {
    map(tag("b"), |_| RunValue::DeadCell)(input)
}

fn alive_cell(input: &[u8]) -> IResult<&[u8], RunValue> {
    map(tag("o"), |_| RunValue::AliveCell)(input)
}

fn line_end(input: &[u8]) -> IResult<&[u8], RunValue> {
    map(tag("$"), |_| RunValue::LineEnd)(input)
}

fn run(input: &[u8]) -> IResult<&[u8], Run> {
    let (input, len) = map(opt(parse_num), |x| x.unwrap_or(1))(input)?;
    let (input, value) = alt((dead_cell, alive_cell, line_end))(input)?;
    Ok((input, Run::new(len, value)))
}

fn whitespace(input: &[u8]) -> IResult<&[u8], &[u8]> {
    is_a(" \t\r\n")(input)
}

fn parse_rle(input: &[u8]) -> IResult<&[u8], Rle> {
    let (input, _) = opt(whitespace)(input)?;
    let (input, runs) = many0(terminated(run, opt(whitespace)))(input)?;
    let (input, _) = tag("!")(input)?;
    Ok((input, Rle { runs }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let glider = " b  ob  $ 2b\n\no\n$3o$$ $ $$$$ 5$ bbb $b$bb$bbb !  foobar";
        let rle = Rle::from_pattern(glider).unwrap();
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
