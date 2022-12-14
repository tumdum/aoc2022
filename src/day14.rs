use anyhow::Result;
use itertools::iterate;
use rustc_hash::FxHashSet as HashSet;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    x: i32,
    y: i32,
}

const MOVE_DIRS: [Pos; 3] = [Pos { x: 0, y: 1 }, Pos { x: -1, y: 1 }, Pos { x: 1, y: 1 }];

impl Pos {
    fn parse(s: &str) -> Self {
        let mut n = s.split(',');
        let x = n.next().unwrap().parse().unwrap();
        let y = n.next().unwrap().parse().unwrap();
        Self { x, y }
    }

    fn dir(&self, other: Self) -> (Self, usize) {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let n = x.abs().max(y.abs()) as usize;
        (
            Self {
                x: x.signum(),
                y: y.signum(),
            },
            n,
        )
    }

    fn add(&self, dir: Self) -> Self {
        Self {
            x: self.x + dir.x,
            y: self.y + dir.y,
        }
    }

    fn next(&self, cave: &[Vec<State>], floor_y: Option<i32>) -> Option<Pos> {
        if let Some(y) = floor_y {
            if (self.y + 1) == y {
                return None;
            }
        }
        MOVE_DIRS
            .into_iter()
            .map(|dir| self.add(dir))
            .find(|p| cave[p.y as usize][p.x as usize] == State::Empty)
    }
}

fn parse(s: &str) -> Vec<Pos> {
    s.split(" -> ").map(Pos::parse).collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Stone,
    Sand,
    Empty,
}

fn simulate(rocks: &HashSet<Pos>, floor_y: Option<i32>) -> usize {
    let start = Pos { x: 500, y: 0 };
    let max_y = rocks.iter().map(|p| p.y).max().unwrap();
    assert!(max_y < 500);
    let width = 500 + max_y + 3;
    let mut cave: Vec<Vec<State>> = vec![vec![State::Empty; width as usize]; (max_y + 3) as usize];
    for pos in rocks {
        cave[pos.y as usize][pos.x as usize] = State::Stone;
    }
    for i in 0.. {
        let mut sand = start;
        while let Some(p) = sand.next(&cave, floor_y) {
            sand = p;
            if floor_y.is_none() && sand.y > max_y {
                return i;
            }
        }
        cave[sand.y as usize][sand.x as usize] = State::Sand;
        if floor_y.is_some() && sand == start {
            return i + 1;
        }
    }
    unreachable!()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let cave: HashSet<Pos> = input
        .lines()
        .map(parse)
        .flat_map(|f| {
            f.windows(2)
                .flat_map(|w| {
                    let (dir, n) = w[1].dir(w[0]);
                    iterate(w[0], move |p| p.add(dir)).take(n + 1)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let s = Instant::now();

    let max_y = cave.iter().map(|p| p.y).max().unwrap();

    let part1 = simulate(&cave, None);
    let part2 = simulate(&cave, Some(max_y + 2));

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(873, part1);
        assert_eq!(24813, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
