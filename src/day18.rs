use anyhow::Result;
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const NEIGHBOURS: [P3; 6] = [
    P3 { x: 1, y: 0, z: 0 },
    P3 { x: -1, y: 0, z: 0 },
    P3 { x: 0, y: 1, z: 0 },
    P3 { x: 0, y: -1, z: 0 },
    P3 { x: 0, y: 0, z: 1 },
    P3 { x: 0, y: 0, z: -1 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

fn can_grow_for(from: P3, time: usize, taken: &HashSet<P3>) -> bool {
    let mut taken = taken.clone();
    let mut todo: VecDeque<P3> = VecDeque::new();
    todo.push_back(from);
    let mut c = 0;
    while let Some(p) = todo.pop_back() {
        if c > time {
            break;
        }
        taken.insert(p);
        for n in p.neighbours() {
            if !taken.contains(&n) {
                c += 1;
                todo.push_back(n);
            }
        }
    }
    c > time
}

fn count_exposed_sides(ps: &[P3]) -> (usize, usize) {
    let points: HashSet<P3> = ps.iter().copied().collect();
    let mut out: HashSet<P3> = Default::default();
    let mut c = 0;
    for p in ps {
        for n in p.neighbours() {
            if !points.contains(&n) {
                c += 1;
                out.insert(n);
            }
        }
    }
    let mut inner: HashSet<P3> = Default::default();
    for p in out {
        if !can_grow_for(p, 2 * ps.len(), &points) {
            inner.insert(p);
        }
    }

    let mut c2 = 0;
    for p in ps {
        for n in p.neighbours() {
            if !points.contains(&n) && !inner.contains(&n) {
                c2 += 1;
            }
        }
    }
    (c, c2)
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
