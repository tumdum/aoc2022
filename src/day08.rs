use anyhow::Result;
use itertools::{
    iproduct, iterate,
    FoldWhile::{Continue, Done},
    Itertools,
};
use std::time::{Duration, Instant};

use crate::input::tokens;
const DIRS: [Pos; 4] = [
    Pos { row: 1, col: 0 },
    Pos { row: -1, col: 0 },
    Pos { row: 0, col: 1 },
    Pos { row: 0, col: -1 },
];

#[derive(Debug, Clone, Copy)]
struct Pos {
    row: isize,
    col: isize,
}

impl Pos {
    fn is_outside(&self, trees: &[Vec<i8>]) -> bool {
        self.row < 0
            || self.col < 0
            || (self.row as usize) >= trees.len()
            || (self.col as usize) >= trees[0].len()
    }

    fn move_by<'a>(&self, dir: Self, trees: &'a [Vec<i8>]) -> impl Iterator<Item = i8> + 'a {
        iterate(*self, move |p| Pos {
            row: p.row + dir.row,
            col: p.col + dir.col,
        })
        .skip(1)
        .map_while(|p| {
            if p.is_outside(trees) {
                None
            } else {
                Some(trees[p.row as usize][p.col as usize])
            }
        })
    }
}

fn scenic_score(trees: &[Vec<i8>], row: usize, col: usize) -> usize {
    let start = Pos {
        row: row as isize,
        col: col as isize,
    };
    let start_h = trees[start.row as usize][start.col as usize];
    DIRS.into_iter().fold(1, |score, dir| {
        score
            * start
                .move_by(dir, trees)
                .fold_while(0, |acc, h| {
                    if h >= start_h {
                        Done(acc + 1)
                    } else {
                        Continue(acc + 1)
                    }
                })
                .into_inner()
    })
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<i8>> = tokens::<String>(input, None)
        .into_iter()
        .map(|l| l.bytes().map(|b| (b - b'0') as i8).collect())
        .collect();

    let s = Instant::now();

    let mut visible = vec![vec![false; input[0].len()]; input.len()];

    let cols = input[0].len();
    let len = input.len();
    let all_rows = 0..len;
    let all_cols = 0..cols;

    let mut mark_visible = |last: &mut i8, row: usize, col: usize| {
        if input[row][col] > *last {
            visible[row][col] = true;
            *last = input[row][col];
        }
    };

    for row in all_rows.clone() {
        let mut last = -1i8;
        all_cols
            .clone()
            .for_each(|col| mark_visible(&mut last, row, col));

        let mut last = -1i8;
        all_cols
            .clone()
            .rev()
            .for_each(|col| mark_visible(&mut last, row, col));
    }
    for col in all_cols.clone() {
        let mut last = -1i8;
        all_rows
            .clone()
            .for_each(|row| mark_visible(&mut last, row, col));

        let mut last = -1i8;
        all_rows
            .clone()
            .rev()
            .for_each(|row| mark_visible(&mut last, row, col));
    }

    let part1 = visible.into_iter().flatten().filter(|b| *b).count();

    let part2 = iproduct!(all_rows, all_cols)
        .map(|(row, col)| scenic_score(&input, row, col))
        .max()
        .unwrap();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(1825, part1);
        assert_eq!(235200, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
