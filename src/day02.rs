use anyhow::{anyhow, Error, Result};
use std::str::FromStr;
use std::time::{Duration, Instant};

const LOSE: char = 'X';
const DRAW: char = 'Y';

#[derive(Debug, PartialEq, Clone, Copy)]
enum Piece {
    Rock,
    Paper,
    Scissors,
}

use Piece::*;

impl Piece {
    fn play(&self, other: &Self) -> i32 {
        if self == other {
            return 3;
        }
        match (self, other) {
            (Rock, Scissors) | (Scissors, Paper) | (Paper, Rock) => 6,
            _ => 0,
        }
    }
    fn value(&self) -> i32 {
        match self {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        }
    }
    fn parse(c: char) -> Self {
        match c {
            'A' | 'X' => Rock,
            'B' | 'Y' => Paper,
            'C' | 'Z' => Scissors,
            _ => panic!(),
        }
    }
    fn select(&self, target: char) -> Piece {
        if target == DRAW {
            *self
        } else if target == LOSE {
            match self {
                Rock => Scissors,
                Scissors => Paper,
                Paper => Rock,
            }
        } else {
            match self {
                Scissors => Rock,
                Paper => Scissors,
                Rock => Paper,
            }
        }
    }
}

impl FromStr for Piece {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::parse(
            s.chars().next().ok_or_else(|| anyhow!("no chars"))?,
        ))
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<(Piece, char)> = input
        .lines()
        .map(|l| {
            let mut tmp = l.split(' ');
            let l = tmp.next().unwrap().parse().unwrap();
            let r = tmp.next().unwrap().chars().next().unwrap();
            (l, r)
        })
        .collect();

    let s = Instant::now();

    let score = |(other, me): (Piece, Piece)| me.value() + me.play(&other);

    let part1: i32 = input
        .iter()
        .map(|(a, b)| (*a, Piece::parse(*b)))
        .map(score)
        .sum();
    let part2: i32 = input
        .iter()
        .map(|(a, b)| (*a, a.select(*b)))
        .map(score)
        .sum();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(11666, part1);
        assert_eq!(12767, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
