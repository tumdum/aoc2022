use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();
    let mut input: Vec<u32> = input
        .split(|s| s.is_empty())
        .map(|s| s.iter().flat_map(|i| i.parse::<u32>()).sum())
        .collect();

    let s = Instant::now();

    let l = input.len();
    let (_, _, top3) = input.select_nth_unstable(l - 4);
    let part1 = *top3
        .iter()
        .max()
        .ok_or_else(|| anyhow!("not enough elements"))?;
    let part2: u32 = top3.iter().sum();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(69912, part1);
        assert_eq!(208180, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
