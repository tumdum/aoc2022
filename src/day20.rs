use anyhow::Result;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

fn run_once(deq: &mut VecDeque<(i64, i64)>, input: &[(i64, i64)]) {
    for (key, _) in input {
        let idx = deq.iter().position(|(orig, _)| *orig == *key).unwrap() as i64;
        let value = deq.remove(idx as usize).unwrap();
        let to_idx = (idx + value.1 as i64).rem_euclid(input.len() as i64 - 1) as i64;
        deq.insert(to_idx as usize, value);
    }
}

fn mix(vals: &[i64], times: usize) -> i64 {
    let input: Vec<(i64, i64)> = vals
        .iter()
        .enumerate()
        .map(|(i, v)| (i as i64, *v))
        .collect();
    let mut deq: VecDeque<(i64, i64)> = input.iter().copied().collect();
    (0..times).for_each(|_| run_once(&mut deq, &input));
    let zero = deq.iter().position(|(_, v)| *v == 0).unwrap();
    deq.iter()
        .cycle()
        .enumerate()
        .take_while(|(i, _)| *i <= zero + 3000)
        .filter(|(i, _)| *i == zero + 1000 || *i == zero + 2000 || *i == zero + 3000)
        .map(|(_, (_, v))| *v)
        .sum::<i64>()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let mut input: Vec<i64> = input.lines().map(|s| s.parse().unwrap()).collect();

    let s = Instant::now();

    let part1 = mix(&input, 1);

    input.iter_mut().for_each(|v| *v *= 811589153);

    let part2 = mix(&input, 10);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(4578, part1);
        assert_eq!(2159638736133, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
