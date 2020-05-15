// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Cell, Error, Result};

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::digit1,
    combinator::{map, map_opt, opt},
    multi::many0,
    sequence::terminated,
    IResult,
};

struct Rle {
    runs: Vec<Run>,
}

struct Run {
    value: RunValue,
    len: u32,
}

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
    Ok((input, Run { value, len }))
}

fn whitespace(input: &[u8]) -> IResult<&[u8], &[u8]> {
    is_a(" \t\r\n")(input)
}

fn parse_rle(input: &[u8]) -> IResult<&[u8], Rle> {
    let (input, runs) = many0(terminated(run, opt(whitespace)))(input)?;
    Ok((input, Rle { runs }))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse() {
//         let glider = "bob$2bo$3o!";
//         let rle = Rle::from_pattern(glider).unwrap();
//         let mut coords: Vec<_> = rle.alive_cells().collect();
//         coords.sort();
//         assert_eq!(coords, &[(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
//     }
// }
