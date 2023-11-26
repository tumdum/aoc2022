use anyhow::Result;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::input::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Noop,
    Addx(i32),
}

impl Op {
    fn len(&self) -> usize {
        match self {
            Self::Addx(_) => 2,
            Self::Noop => 1,
        }
    }
}

impl FromStr for Op {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut s = s.split(' ');
        match s.next().unwrap() {
            "addx" => Ok(Self::Addx(s.next().unwrap().parse().unwrap())),
            "noop" => Ok(Self::Noop),
            s => Err(anyhow::anyhow!("unexpected '{s}'")),
        }
    }
}

fn print(screen: &[char]) {
    for line in screen.chunks(40) {
        let s: String = line.iter().copied().collect();
        println!("{s}");
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let mut ops: Vec<Op> = tokens(input, Some("\n"));

    let s = Instant::now();

    let mut current_op_cyc = 0;
    let mut current_op: Option<Op> = None;
    let mut x = 1;
    let mut xs = vec![];
    ops.reverse();
    while !ops.is_empty() || current_op.is_some() {
        match current_op {
            None => {
                current_op = ops.pop();
                current_op_cyc = 1;
            }

            Some(op) if op.len() == current_op_cyc => {
                if let Some(Op::Addx(n)) = current_op {
                    x += n;
                }
                current_op = ops.pop();
                current_op_cyc = 1;
            }
            _ => {
                current_op_cyc += 1;
            }
        }
        xs.push(x);
    }

    let idxs = [20i32, 60, 100, 140, 180, 220];
    let part1 = idxs
        .into_iter()
        .map(|i| xs[(i - 1) as usize] * i)
        .sum::<i32>();

    let mut screen = vec!['X'; 240];

    for c in 0i32..240 {
        let cc = c % 40;
        screen[c as usize] = if (xs[c as usize] - cc).abs() <= 1 {
            '#'
        } else {
            ' '
        };
    }

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(15360, part1);
        // assert_eq!(2585, part2);
    }
    if output {
        println!("\t{}", part1);
        print(&screen); // part 2
    }
    Ok(e)
}
