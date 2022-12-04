use crate::U8Set;
use anyhow::Result;
use std::time::{Duration, Instant};

fn score(b: u8) -> u64 {
    if b < b'a' {
        let b = b - b'A' + 1;
        b as u64 + 26
    } else {
        let b = b - b'a' + 1;
        b as u64
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();

    let s = Instant::now();

    let part1: u64 = input
        .iter()
        .flat_map(|v| {
            let len = v.len() / 2;
            let l: U8Set = v[..len].iter().collect();
            let r: U8Set = v[len..].iter().collect();
            l.intersection(&r).iter().next()
        })
        .map(score)
        .sum();

    let part2: u64 = input
        .chunks(3)
        .flat_map(|c| {
            let a: U8Set = c[0].iter().collect();
            let b: U8Set = c[1].iter().collect();
            let c: U8Set = c[2].iter().collect();
            a.intersection(&b).intersection(&c).iter().next()
        })
        .map(score)
        .sum();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(7917, part1);
        assert_eq!(2585, part2);
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
    fn score_test() {
        assert_eq!(score(b'a'), 1);
        assert_eq!(score(b'z'), 26);
        assert_eq!(score(b'A'), 27);
        assert_eq!(score(b'Z'), 52);
    }
}
