use crate::U8Set;
use anyhow::Result;
use std::time::{Duration, Instant};

fn all_diff(v: &[u8]) -> bool {
    let mut seen = U8Set::default();
    for c in v {
        if seen.insert(*c) {
            return false;
        }
    }
    true
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<_> = input.trim().bytes().collect();

    let s = Instant::now();

    let len = 4;
    let part1 = input
        .windows(len)
        .enumerate()
        .find(|(_, w)| all_diff(w))
        .unwrap()
        .0
        + len;

    let len = 14;
    let part2 = input
        .windows(len)
        .enumerate()
        .find(|(_, w)| all_diff(w))
        .unwrap()
        .0
        + len;

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(1912, part1);
        assert_eq!(2122, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
