use anyhow::Result;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Move {
    from: usize,
    to: usize,
    count: usize,
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<_> = input.lines().collect();
    let mut input = input.split(|l| l.is_empty());
    let stacks: Vec<Vec<u8>> = input
        .next()
        .unwrap()
        .iter()
        .map(|l| l.bytes().collect())
        .collect();
    let mut stacks: Vec<Vec<u8>> = stacks
        .last()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, column_name)| **column_name != b' ')
        .map(|(column_idx, _)| {
            stacks
                .iter()
                .map(|l| l[column_idx])
                .filter(|c| c.is_ascii_alphabetic())
                .rev()
                .collect()
        })
        .collect();
    let moves: Vec<Move> = input
        .next()
        .unwrap()
        .iter()
        .map(|l| {
            let mut l = l.split(' ').flat_map(|s| s.parse::<usize>());
            let count = l.next().unwrap();
            let from = l.next().unwrap();
            let to = l.next().unwrap();
            Move { from, to, count }
        })
        .collect();

    let s = Instant::now();

    let part1: String = {
        let mut stacks = stacks.clone();
        for m in &moves {
            for _ in 0..m.count {
                let tmp = stacks[m.from - 1].pop().unwrap();
                stacks[m.to - 1].push(tmp);
            }
        }
        stacks.iter().map(|s| *s.last().unwrap() as char).collect()
    };

    for m in &moves {
        let cut_point = stacks[m.from - 1].len() - m.count;
        let to_move = stacks[m.from - 1][cut_point..].to_vec();
        stacks[m.from - 1].truncate(cut_point);
        stacks[m.to - 1].extend_from_slice(&to_move);
    }
    let part2: String = stacks.iter().map(|s| *s.last().unwrap() as char).collect();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!("SHMSDGZVC", part1);
        assert_eq!("VRZGHDFBQ", part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
