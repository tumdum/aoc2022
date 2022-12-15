use anyhow::Result;
use itertools::Itertools;
use smallvec::{smallvec, SmallVec};
use std::collections::HashSet;
use std::time::{Duration, Instant};

type V<T> = SmallVec<[T; 8]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    row: i32,
    col: i32,
}

impl Pos {
    fn dist(self, other: Self) -> i32 {
        (self.row - other.row).abs() + (self.col - other.col).abs()
    }
}

fn sorted_overlap((_, max1): (i32, i32), (min2, _): (i32, i32)) -> bool {
    min2 <= max1 || max1 + 1 == min2
}

fn parse(s: &str) -> (Pos, Pos) {
    let s = s.split(' ').collect_vec();
    let parse = |p, s, i: &str| {
        i.strip_prefix(p)
            .unwrap()
            .strip_suffix(s)
            .unwrap()
            .parse()
            .unwrap()
    };
    let sx = parse("x=", ",", s[2]);
    let sy = parse("y=", ":", s[3]);
    let bx = parse("x=", ",", s[8]);
    let by = parse("y=", "", s[9]);
    let s = Pos { row: sy, col: sx };
    let b = Pos { row: by, col: bx };
    (s, b)
}

fn min_max_in_range_for(s: Pos, b: Pos, row: i32) -> Option<(i32, i32)> {
    let dist = s.dist(b);
    let off = dist - (s.row - row).abs();
    if off < 0 {
        None
    } else {
        Some((s.col - off, s.col + off))
    }
}

fn intervals_for_row(row: i32, input: &[(Pos, Pos)]) -> V<(i32, i32)> {
    let mut intervals: V<(i32, i32)> = input
        .iter()
        .flat_map(|(s, b)| min_max_in_range_for(*s, *b, row))
        .collect();
    intervals.sort_unstable_by_key(|(min, _)| *min);

    let mut intervals_merged: V<(i32, i32)> = smallvec![intervals[0]];
    for next in &intervals[1..] {
        let last_idx = intervals_merged.len() - 1;
        let last = intervals_merged[last_idx];
        if sorted_overlap(last, *next) {
            if next.1 > last.1 {
                intervals_merged[last_idx].1 = next.1;
            }
        } else {
            intervals_merged.push(*next);
        }
    }
    intervals_merged
}

fn find_freq(input: &[(Pos, Pos)], max_row: i32) -> i64 {
    use rayon::prelude::*;
    (0..=max_row)
        .rev()
        .par_bridge()
        .map(|row| (row, intervals_for_row(row, input)))
        .find_any(|(_, int)| int.len() > 1)
        .map(|(row, int)| (int[0].1 as i64 + 1) * 4000000 + row as i64)
        .unwrap()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<(Pos, Pos)> = input.lines().map(parse).collect();

    let s = Instant::now();

    let row = 2000000;
    let b_in_row: HashSet<_> = input.iter().map(|v| v.1).filter(|v| v.row == row).collect();
    let part1 = intervals_for_row(row, &input)
        .iter()
        .map(|(min, max)| max - min + 1)
        .sum::<i32>() as usize
        - b_in_row.len();
    let part2 = find_freq(&input, 4000000);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(5083287, part1);
        assert_eq!(13134039205729, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
