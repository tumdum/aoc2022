use anyhow::Result;
use itertools::iproduct;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::{Duration, Instant};

use crate::dijkstra::dijkstra;
use crate::input::tokens;

const NEIGHBOURS_OFF: [Pos; 4] = [
    Pos { row: 1, col: 0 },
    Pos { row: -1, col: 0 },
    Pos { row: 0, col: 1 },
    Pos { row: 0, col: -1 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pos {
    row: i32,
    col: i32,
}

impl Pos {
    fn get_mut<'a>(&self, m: &'a mut [Vec<u8>]) -> Option<&'a mut u8> {
        if self.is_in_bounds(m) {
            Some(&mut m[self.row as usize][self.col as usize])
        } else {
            None
        }
    }
    fn get<T: Copy>(&self, m: &[Vec<T>]) -> Option<T> {
        if self.is_in_bounds(m) {
            Some(m[self.row as usize][self.col as usize])
        } else {
            None
        }
    }

    fn is_in_bounds<T>(&self, m: &[Vec<T>]) -> bool {
        self.row >= 0
            && self.row < m.len() as i32
            && self.col >= 0
            && self.col < m[self.row as usize].len() as i32
    }

    fn get_neighbours<'a>(&self, m: &'a [Vec<u8>]) -> impl Iterator<Item = (Pos, u8)> + 'a {
        let this = *self;
        NEIGHBOURS_OFF
            .into_iter()
            .map(move |off| this.move_by(off))
            .filter_map(move |pos| pos.get(m).map(|h| (pos, h)))
    }

    fn move_by(&self, offset: Pos) -> Pos {
        Pos {
            row: self.row + offset.row,
            col: self.col + offset.col,
        }
    }
}

fn find(m: &[Vec<u8>], e: u8) -> Option<Pos> {
    for (row_idx, row) in m.iter().enumerate() {
        for (col_idx, element) in row.iter().enumerate() {
            if *element == e {
                return Some(Pos {
                    row: row_idx as i32,
                    col: col_idx as i32,
                });
            }
        }
    }
    None
}

fn can_move(from: u8, to: u8) -> bool {
    let from = from as i8;
    let to = to as i8;
    (to - from) <= 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    path_len: u32,
    height: u8,
    next: Pos,
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let mut m: Vec<Vec<u8>> = tokens::<String>(input, None)
        .into_iter()
        .map(|l| l.bytes().collect())
        .collect();

    let s = Instant::now();

    let start = find(&m, b'S').unwrap();
    let target = find(&m, b'E').unwrap();

    *start.get_mut(&mut m).unwrap() = b'a';
    *target.get_mut(&mut m).unwrap() = b'z';

    let can_move_inv = |a, b| can_move(b, a);
    let neighbours_of = |p: &Pos| {
        let from: u8 = p.get(&m).unwrap();
        let n: Vec<_> = p.get_neighbours(&m).collect();
        n.into_iter()
            .filter(|(_, h)| can_move_inv(from, *h))
            .map(|(next, _)| (next, 1))
            .collect()
    };
    let (cost, _prev) = dijkstra(target, neighbours_of);
    let part1 = *cost.get(&start).unwrap();

    let part2 = *iproduct!(0..m.len(), 0..m[0].len())
        .filter(|(row, col)| m[*row][*col] == b'a')
        .filter_map(|(row, col)| {
            cost.get(&Pos {
                row: row as i32,
                col: col as i32,
            })
        })
        .min()
        .unwrap();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(408, part1);
        assert_eq!(399, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_move_test() {
        assert!(can_move(b'z', b'a'));
        assert!(can_move(b'm', b'n'));
        assert!(!can_move(b'm', b'o'));
    }
}
