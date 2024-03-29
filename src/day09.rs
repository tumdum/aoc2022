use anyhow::{anyhow, Error, Result};
use rustc_hash::FxHashSet as HashSet;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::input::token_groups;

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
struct Pos {
    x: i16,
    y: i16,
}

impl Pos {
    fn is_touching(&self, other: &Self) -> bool {
        (self.x - other.x).abs() <= 1 && (self.y - other.y).abs() <= 1
    }

    fn move_by(&self, delta: Self) -> Self {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
        }
    }

    fn move_to(&self, target: &Self) -> Self {
        if self.is_touching(target) {
            return *self;
        }
        let diff = Pos {
            x: (target.x - self.x).signum(),
            y: (target.y - self.y).signum(),
        };
        self.move_by(diff)
    }

    fn move_dir(&self, dir: &Dir) -> Self {
        self.move_by(DIR_2_DELTA[*dir as usize])
    }
}

#[derive(Debug, Clone, Copy)]
enum Dir {
    R = 0,
    U = 1,
    L = 2,
    D = 3,
}

const DIR_2_DELTA: [Pos; 4] = [
    Pos { x: 1, y: 0 },
    Pos { x: 0, y: 1 },
    Pos { x: -1, y: 0 },
    Pos { x: 0, y: -1 },
];

impl FromStr for Dir {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "R" => Ok(Self::R),
            "U" => Ok(Self::U),
            "L" => Ok(Self::L),
            "D" => Ok(Self::D),
            s => Err(anyhow!("unexpected string: '{s}'")),
        }
    }
}

fn simulate_and_find_tail_positions(mut rope: Vec<Pos>, moves: &[(Dir, usize)]) -> HashSet<Pos> {
    let mut seen = HashSet::default();
    seen.insert(*rope.last().unwrap());

    for (dir, count) in moves {
        for _ in 0..*count {
            rope[0] = rope[0].move_dir(dir);
            for i in 1..rope.len() {
                rope[i] = rope[i].move_to(&rope[i - 1]);
            }
            seen.insert(*rope.last().unwrap());
        }
    }
    seen
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<(Dir, usize)> = token_groups::<String>(input, "\n", Some(" "))
        .into_iter()
        .map(|l| {
            let dir = l[0].parse().unwrap();
            let count = l[1].parse().unwrap();
            (dir, count)
        })
        .collect();

    let s = Instant::now();

    let rope = vec![Pos { x: 0, y: 0 }; 2];
    let part1 = simulate_and_find_tail_positions(rope, &input).len();

    let rope = vec![Pos { x: 0, y: 0 }; 10];
    let part2 = simulate_and_find_tail_positions(rope, &input).len();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(5735, part1);
        assert_eq!(2478, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
