use anyhow::Result;
use itertools::Itertools;
use rustc_hash::FxHashSet as HashSet;
use smallvec::{smallvec, SmallVec};
use std::time::{Duration, Instant};

type V<T> = SmallVec<[T; 8]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    row: isize,
    col: isize,
}

impl Pos {
    fn dist(self, other: Self) -> (isize, isize) {
        ((self.row - other.row).abs(), (self.col - other.col).abs())
    }
}

fn sorted_overlap((min1, max1): (isize, isize), (min2, _): (isize, isize)) -> bool {
    assert!(min1 <= min2);
    min2 <= max1 || max1 + 1 == min2
}

fn parse(s: &str) -> (Pos, Pos) {
    let s = s.split(' ').collect_vec();
    let sx: isize = s[2]
        .strip_prefix("x=")
        .unwrap()
        .strip_suffix(',')
        .unwrap()
        .parse()
        .unwrap();
    let sy: isize = s[3]
        .strip_prefix("y=")
        .unwrap()
        .strip_suffix(':')
        .unwrap()
        .parse()
        .unwrap();
    let bx: isize = s[8]
        .strip_prefix("x=")
        .unwrap()
        .strip_suffix(',')
        .unwrap()
        .parse()
        .unwrap();
    let by: isize = s[9].strip_prefix("y=").unwrap().parse().unwrap();
    let s = Pos { row: sy, col: sx };
    let b = Pos { row: by, col: bx };
    (s, b)
}

fn min_max_in_range_for(s: Pos, b: Pos, row: isize) -> Option<(isize, isize)> {
    let (h, w) = s.dist(b);
    let dist = h + w;
    let off = dist - (s.row - row).abs();
    if off < 0 {
        None
    } else {
        Some((s.col - off, s.col + off))
    }
}

fn intervals_for_row(row: isize, input: &[(Pos, Pos)]) -> V<(isize, isize)> {
    let mut intervals: V<(isize, isize)> = smallvec![];
    for (s, b) in input {
        if let Some((min, max)) = min_max_in_range_for(*s, *b, row) {
            intervals.push((min, max));
        }
        if s.row == row {
            intervals.push((s.col, s.col));
        }
    }
    intervals.sort_unstable_by_key(|(min, _)| *min);

    let mut intervals_merged: V<(isize, isize)> = smallvec![intervals[0]];
    let mut next = 1;
    while next < intervals.len() {
        let last_idx = intervals_merged.len() - 1;
        let last = intervals_merged[last_idx];
        if sorted_overlap(last, intervals[next]) {
            if intervals[next].1 > intervals_merged[last_idx].1 {
                intervals_merged[last_idx].1 = intervals[next].1;
            }
        } else {
            intervals_merged.push(intervals[next]);
        }
        next += 1;
    }
    intervals_merged
}

fn find_freq(input: &[(Pos, Pos)], max_row: isize) -> isize {
    for row in 0..=max_row {
        let intervals = intervals_for_row(row, input);
        if intervals.len() > 1 {
            return (intervals[0].1 + 1) * 4000000 + row;
        }
    }
    unreachable!()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<(Pos, Pos)> = input.lines().map(parse).collect();

    let s = Instant::now();

    let row = 2000000;
    let b_in_row: HashSet<_> = input.iter().map(|v| v.1).filter(|v| v.row == row).collect();
    let part1 = intervals_for_row(row, &input)
        .iter()
        .map(|(min, max)| max - min + 1)
        .sum::<isize>() as usize
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
