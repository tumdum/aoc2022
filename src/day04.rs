use anyhow::Result;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Range {
    first: i64,
    last: i64,
}

impl Range {
    fn parse(s: &str) -> Self {
        let mut s = s.split('-');
        let first = s.next().unwrap().parse().unwrap();
        let last = s.next().unwrap().parse().unwrap();
        Self { first, last }
    }

    fn includes(&self, other: &Self) -> bool {
        self.first >= other.first && self.last <= other.last
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.first >= other.first && self.first <= other.last
    }
}

fn includes(a: &Range, b: &Range) -> bool {
    a.includes(b) || b.includes(a)
}

fn overlaps(a: &Range, b: &Range) -> bool {
    a.overlaps(b) || b.overlaps(a)
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<_> = input
        .lines()
        .map(|l| {
            let mut l = l.split(',');
            let a = Range::parse(l.next().unwrap());
            let b = Range::parse(l.next().unwrap());
            (a, b)
        })
        .collect();

    let s = Instant::now();

    let part1 = input.iter().filter(|(a, b)| includes(a, b)).count();

    let part2 = input.iter().filter(|(a, b)| overlaps(a, b)).count();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(477, part1);
        assert_eq!(830, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
