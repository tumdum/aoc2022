use anyhow::Result;
use itertools::iproduct;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

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

impl State {
    fn new(next: Pos, height: u8, path_len: u32) -> Self {
        Self {
            next,
            height,
            path_len,
        }
    }
}

fn find_path_len(
    start: Pos,
    target: Pos,
    m: &[Vec<u8>],
    pred: impl Fn(u8, u8) -> bool,
) -> Vec<Vec<Option<(Pos, u32)>>> {
    let mut best: Vec<Vec<Option<(Pos, u32)>>> = vec![vec![None; m[0].len()]; m.len()];
    // Instead of Dijkstra that would use here BinaryHeap, use VecDeque to get
    // simple bfs that make this problem run much faster.
    let mut todo = VecDeque::with_capacity(64);
    todo.push_back(State::new(start, start.get(m).unwrap(), 0));
    best[start.row as usize][start.col as usize] = Some((start, 0));
    'out: while let Some(state) = todo.pop_front() {
        for (pos, h) in state
            .next
            .get_neighbours(m)
            .filter(|(_, h)| pred(state.height, *h))
        {
            let next_path_len = state.path_len + 1;
            if best[pos.row as usize][pos.col as usize]
                .map(|(_, h)| h)
                .unwrap_or(u32::max_value())
                <= next_path_len
            {
                continue;
            }
            best[pos.row as usize][pos.col as usize] = Some((state.next, next_path_len));
            todo.push_back(State::new(pos, h, next_path_len));
            if pos == target {
                break 'out;
            }
        }
    }

    best
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let mut m: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();

    let s = Instant::now();

    let start = find(&m, b'S').unwrap();
    let target = find(&m, b'E').unwrap();

    *start.get_mut(&mut m).unwrap() = b'a';
    *target.get_mut(&mut m).unwrap() = b'z';

    let can_move_inv = |a, b| can_move(b, a);
    let best = find_path_len(target, Pos { row: -10, col: -10 }, &m, can_move_inv);
    let part1 = start.get(&best).unwrap().unwrap().1 as usize;
    let part2 = iproduct!(0..m.len(), 0..m[0].len())
        .filter(|(row, col)| m[*row][*col] == b'a')
        .filter_map(|(row, col)| best[row][col].map(|(_, l)| l))
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
