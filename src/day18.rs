use anyhow::Result;
use itertools::Itertools;
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

const NEIGHBOURS: [P3; 6] = [
    P3 { x: 1, y: 0, z: 0 },
    P3 { x: -1, y: 0, z: 0 },
    P3 { x: 0, y: 1, z: 0 },
    P3 { x: 0, y: -1, z: 0 },
    P3 { x: 0, y: 0, z: 1 },
    P3 { x: 0, y: 0, z: -1 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct P3 {
    x: i32,
    y: i32,
    z: i32,
}

impl P3 {
    fn neighbours(&'_ self) -> impl Iterator<Item = P3> + '_ {
        NEIGHBOURS.into_iter().map(|dir| self.add(dir))
    }
    fn add(self, other: Self) -> Self {
        P3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

fn parse(s: &str) -> P3 {
    let mut v = s.split(',').map(|s| s.parse().unwrap());
    P3 {
        x: v.next().unwrap(),
        y: v.next().unwrap(),
        z: v.next().unwrap(),
    }
}

fn grow_from(
    from: P3,
    taken: &HashSet<P3>,
    (x, y, z): (
        RangeInclusive<i32>,
        RangeInclusive<i32>,
        RangeInclusive<i32>,
    ),
) -> HashSet<P3> {
    let mut seen: HashSet<P3> = HashSet::default();
    let mut todo: VecDeque<P3> = VecDeque::new();
    todo.push_back(from);
    seen.insert(from);
    while let Some(p) = todo.pop_back() {
        if !x.contains(&p.x) || !y.contains(&p.y) || !z.contains(&p.z) {
            continue;
        }
        for n in p.neighbours() {
            if !taken.contains(&n) && !seen.contains(&n) {
                todo.push_back(n);
                seen.insert(n);
            }
        }
    }
    seen
}

fn count_exposed_sides(ps: &[P3]) -> (usize, usize) {
    let (min_x, max_x) = ps.iter().map(|p| p.x).minmax().into_option().unwrap();
    let (min_y, max_y) = ps.iter().map(|p| p.y).minmax().into_option().unwrap();
    let (min_z, max_z) = ps.iter().map(|p| p.z).minmax().into_option().unwrap();

    let points: HashSet<P3> = ps.iter().copied().collect();
    let mut air_around_stones: HashSet<P3> = Default::default();
    let mut sides_exposed_to_air = 0;
    for p in ps {
        for n in p.neighbours() {
            if !points.contains(&n) {
                sides_exposed_to_air += 1;
                air_around_stones.insert(n);
            }
        }
    }

    let outside = P3 {
        x: min_x - 1,
        y: min_y - 1,
        z: min_z - 1,
    };

    let outside_points = grow_from(
        outside,
        &points,
        (
            ((min_x - 1)..=(max_x + 1)),
            ((min_y - 1)..=(max_y + 1)),
            ((min_z - 1)..=(max_z + 1)),
        ),
    );

    let without_inner_air = ps
        .iter()
        .flat_map(|p| p.neighbours())
        .filter(|p| !points.contains(&p) && outside_points.contains(p))
        .count();

    (sides_exposed_to_air, without_inner_air)
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<P3> = input.lines().map(parse).collect();

    let s = Instant::now();

    let (part1, part2) = count_exposed_sides(&input);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(3610, part1);
        assert_eq!(2082, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
