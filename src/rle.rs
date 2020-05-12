use crate::Cell;
use crate::{Error, Result};

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;

#[derive(Debug, Parser)]
#[grammar = "rle.pest"]
struct RleParser;

pub fn parse_file(pattern: &str) -> CellGrid {
    todo!()
}

#[derive(Clone, Copy, Debug)]
struct Run {
    len: usize,
    cell: Cell,
}

#[derive(Clone, Debug)]
pub struct Rle {
    runs: Vec<Vec<Run>>,
}

impl Rle {
    pub fn from_pattern(pattern: &str) -> Result<Self> {
        let mut pairs: Pairs<Rule> = RleParser::parse(Rule::Pattern, pattern)
            .map_err(|_| Error::RleParse)?;
        let pattern: Pair<Rule> = pairs.next().unwrap();

        let mut runs = vec![vec![]];
        let mut row = 0;

        for pair in pattern.into_inner() {
            match pair.as_rule() {
                Rule::PatternEnd => break,

                Rule::LineEnd => {
                    runs.push(vec![]);
                    row += 1;
                },

                Rule::Run => {
                },

                _ => unreachable!(),
            }
        };

        Ok(Self { runs })
    }
}

fn parse_pattern(pattern: &str) -> Result<Rle> {
    let root = RleParser::parse(Rule::Pattern, pattern)
        .unwrap()
        .next()
        .unwrap();

    todo!()
}

pub struct CellGrid {
    pub cells: Vec<Vec<Cell>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn parse() {
        let root = RleParser::parse(Rule::Pattern, "b      ob$\n2b o$3o!")
            .unwrap()
            .next()
            .unwrap();

        let mut cells: Vec<Vec<Cell>> = vec![];
        cells.push(vec![]);
        let mut row = 0;

        for pair in root.into_inner() {
            match pair.as_rule() {
                Rule::Run => {
                    let mut foo = pair.into_inner();

                    let (count, cell) = match (foo.next(), foo.next()) {
                        (Some(cell), None) => (1, cell),
                        (Some(count), Some(cell)) => (count.as_str().parse().unwrap(), cell),
                        _ => unreachable!(),
                    };

                    let cell: Cell = match cell.as_rule() {
                        Rule::Dead => Cell::Dead,
                        Rule::Alive => Cell::Alive,
                        _ => unreachable!(),
                    };

                    cells[row].extend(std::iter::repeat(cell).take(count));
                }

                Rule::PatternEnd => {
                    break;
                }

                Rule::LineEnd => {
                    row += 1;
                    cells.push(vec![]);
                }

                _ => unreachable!(),
            }
        }

        for row in cells {
            println!("{:?}", row);
        }

        println!("{:?}", Rle::from_pattern("b      ob$\n2b o$3o!"));
    }
}
