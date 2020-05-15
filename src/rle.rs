// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{Cell, Error, Result};

use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "rle.pest"]
struct RleParser;

#[derive(Clone, Copy, Debug)]
struct Run(usize, Cell);

#[derive(Clone, Debug)]
pub struct Rle {
    runs: Vec<Vec<Run>>,
}

impl Rle {
    pub fn from_pattern(pattern: &str) -> Result<Self> {
        let pattern: Pair<Rule> = RleParser::parse(Rule::Pattern, pattern)
            .map_err(|_| Error::RleParse)?
            .next()
            .unwrap();

        let mut runs: Vec<Vec<Run>> = vec![vec![]];
        let mut row = 0;

        for pair in pattern.into_inner() {
            match pair.as_rule() {
                Rule::PatternEnd => break,

                Rule::LineEnd => {
                    runs.push(vec![]);
                    row += 1;
                }

                Rule::Run => {
                    let mut run_pairs = pair.into_inner();

                    let first_elem = run_pairs.next().unwrap();
                    let maybe_second_elem = run_pairs.next();

                    let run = match (
                        first_elem.as_rule(),
                        maybe_second_elem.as_ref().map(|x| x.as_rule()),
                    ) {
                        (Rule::Cell, None) => {
                            let cell = match first_elem.into_inner().next().unwrap().as_rule() {
                                Rule::Dead => Cell::Dead,
                                Rule::Alive => Cell::Alive,
                                _ => unreachable!(),
                            };
                            Run(1, cell)
                        }

                        (Rule::Number, Some(Rule::Cell)) => {
                            let cell = match maybe_second_elem
                                .unwrap()
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_rule()
                            {
                                Rule::Dead => Cell::Dead,
                                Rule::Alive => Cell::Alive,
                                _ => unreachable!(),
                            };
                            let number = first_elem.as_str().parse().unwrap();
                            Run(number, cell)
                        }

                        _ => unreachable!(),
                    };

                    runs[row].push(run);
                }

                _ => unreachable!(),
            }
        }

        Ok(Self { runs })
    }

    pub fn alive_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.runs.iter().enumerate().flat_map(|(row, row_vec)| {
            let mut col = 0;
            row_vec
                .iter()
                .filter_map(move |run| {
                    let Run(len, cell) = run;
                    let indices = (col..(col + len)).map(move |c| (row, c));
                    col += len;
                    match cell {
                        Cell::Dead => None,
                        Cell::Alive => Some(indices),
                    }
                })
                .flatten()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let glider = "bob$2bo$3o!";
        let rle = Rle::from_pattern(glider).unwrap();
        let mut coords: Vec<_> = rle.alive_cells().collect();
        coords.sort();
        assert_eq!(coords, &[(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
    }
}
