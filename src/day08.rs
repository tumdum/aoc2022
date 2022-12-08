use anyhow::Result;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Pos {
    row: isize,
    col: isize,
}

impl Pos {
    fn add(&mut self, other: &Self) {
        self.row += other.row;
        self.col += other.col;
    }

    fn is_outside(&self, trees: &[Vec<i8>]) -> bool {
        self.row < 0
            || self.col < 0
            || (self.row as usize) >= trees.len()
            || (self.col as usize) >= trees[0].len()
    }

    fn get(&self, trees: &[Vec<i8>]) -> Option<i8> {
        if self.is_outside(trees) {
            None
        } else {
            Some(trees[self.row as usize][self.col as usize])
        }
    }
}

fn scenic_score(trees: &[Vec<i8>], row: usize, col: usize) -> usize {
    let start = Pos {
        row: row as isize,
        col: col as isize,
    };
    let start_h = trees[start.row as usize][start.col as usize];
    let dirs = [
        Pos { row: 1, col: 0 },
        Pos { row: -1, col: 0 },
        Pos { row: 0, col: 1 },
        Pos { row: 0, col: -1 },
    ];
    let mut score_acc = 1;
    for dir in dirs {
        let mut c = 0;
        let mut cur = start;
        loop {
            cur.add(&dir);
            if let Some(h) = cur.get(&trees) {
                c += 1;
                if h >= start_h {
                    break;
                }
            } else {
                break;
            }
        }
        score_acc *= c;
    }
    score_acc
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<i8>> = input
        .lines()
        .map(|l| l.bytes().map(|b| (b - b'0') as i8).collect())
        .collect();

    let s = Instant::now();

    let mut visible: Vec<Vec<bool>> = input
        .iter()
        .map(|l| l.iter().map(|_| false).collect())
        .collect();

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

    let part1 = visible
        .into_iter()
        .flat_map(|row| row)
        .filter(|b| *b)
        .count();

    let part2 = all_rows
        .flat_map(|row| {
            let row = row;
            all_cols
                .clone()
                .map(|col| scenic_score(&input, row, col))
                .collect::<Vec<_>>()
        })
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
