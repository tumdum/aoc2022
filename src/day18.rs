use anyhow::Result;
use itertools::Itertools;
use rustc_hash::FxHashSet as HashSet;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

type VSet = Vec<Vec<Vec<bool>>>;

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
    fn dist(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
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

fn get(m: &[Vec<Vec<bool>>], P3 { x, y, z }: P3, default: bool) -> bool {
    m.get(x as usize)
        .and_then(|v| v.get(y as usize))
        .and_then(|v| v.get(z as usize))
        .copied()
        .unwrap_or(default)
}

fn set(m: &mut [Vec<Vec<bool>>], P3 { x, y, z }: P3) {
    if x < 0 || x as usize >= m.len() {
        return;
    }
    if y < 0 || y as usize >= m[x as usize].len() {
        return;
    }
    if z < 0 || z as usize >= m[x as usize][y as usize].len() {
        return;
    }
    m[x as usize][y as usize][z as usize] = true;
}

fn count_exposed_sides(ps: &[P3]) -> (usize, usize) {
    let (min_x, max_x) = ps.iter().map(|p| p.x).minmax().into_option().unwrap();
    let (min_y, max_y) = ps.iter().map(|p| p.y).minmax().into_option().unwrap();
    let (min_z, max_z) = ps.iter().map(|p| p.z).minmax().into_option().unwrap();

    let mut points =
        vec![vec![vec![false; 1 + max_z as usize]; 1 + max_y as usize]; 1 + max_x as usize];
    for p in ps {
        points[p.x as usize][p.y as usize][p.z as usize] = true;
    }

    let mut air_around_stones: HashSet<P3> = Default::default();
    let mut sides_exposed_to_air = 0;
    for p in ps {
        for n in p.neighbours() {
            if !get(&points, n, false) {
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

    let mut inner =
        vec![vec![vec![false; 1 + max_z as usize]; 1 + max_y as usize]; 1 + max_x as usize];
    air_around_stones
        .iter()
        .filter(|p| !find_path(**p, outside, &points))
        .for_each(|p| set(&mut inner, *p));

    let without_inner_air = ps
        .iter()
        .flat_map(|p| p.neighbours())
        .filter(|p| !get(&points, *p, false) && !get(&inner, *p, false))
        .count();

    (sides_exposed_to_air, without_inner_air)
}

fn find_path(from: P3, to: P3, taken: &VSet) -> bool {
    let mut todo: BinaryHeap<Reverse<(i32, P3)>> = Default::default();
    let mut seen = taken.clone();
    todo.push(Reverse((from.dist(to), from)));
    while let Some(Reverse((_, p))) = todo.pop() {
        if p == to {
            return true;
        }
        for n in p.neighbours() {
            if !get(&seen, n, false) {
                set(&mut seen, n);
                todo.push(Reverse((n.dist(to), n)));
            }
        }
    }
    false
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
