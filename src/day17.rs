use anyhow::Result;
use rustc_hash::FxHashSet as HashSet;
use smallvec::{smallvec, SmallVec};
use std::time::{Duration, Instant};

type V<T> = SmallVec<[T; 5]>;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Pos {
    row: i64,
    col: i64,
}

impl Pos {
    fn add(&mut self, dir: Pos) {
        self.row += dir.row;
        self.col += dir.col;
    }
}

const UP: Pos = Pos { row: 1, col: 0 };
const DOWN: Pos = Pos { row: -1, col: 0 };
const RIGHT: Pos = Pos { row: 0, col: 1 };

fn parse(b: u8) -> Pos {
    if b == b'<' {
        Pos { row: 0, col: -1 }
    } else if b == b'>' {
        Pos { row: 0, col: 1 }
    } else {
        panic!()
    }
}

#[derive(Debug, Clone)]
struct Shape(V<Pos>);

impl Shape {
    fn apply_n(&mut self, dir: Pos, times: i64) {
        let dir = Pos {
            row: dir.row * times,
            col: dir.col * times,
        };
        self.0.iter_mut().for_each(|p| p.add(dir));
    }
    fn apply(&mut self, dir: Pos) {
        self.0.iter_mut().for_each(|p| p.add(dir));
    }

    fn lowest(&self) -> i64 {
        self.0.iter().map(|p| p.row).min().unwrap()
    }

    fn left_most(&self) -> i64 {
        self.0.iter().map(|p| p.col).min().unwrap()
    }
}

fn line() -> Shape {
    Shape(smallvec![
        Pos { row: 0, col: 0 },
        Pos { row: 0, col: 1 },
        Pos { row: 0, col: 2 },
        Pos { row: 0, col: 3 },
    ])
}
fn cross() -> Shape {
    Shape(smallvec![
        Pos { row: 0, col: 0 },
        Pos { row: 0, col: 1 },
        Pos { row: 1, col: 1 },
        Pos { row: -1, col: 1 },
        Pos { row: 0, col: 2 },
    ])
}
fn l() -> Shape {
    Shape(smallvec![
        Pos { row: 0, col: 0 },
        Pos { row: 0, col: 1 },
        Pos { row: 0, col: 2 },
        Pos { row: 1, col: 2 },
        Pos { row: 2, col: 2 },
    ])
}

fn line_down() -> Shape {
    Shape(smallvec![
        Pos { row: 0, col: 0 },
        Pos { row: -1, col: 0 },
        Pos { row: -2, col: 0 },
        Pos { row: -3, col: 0 },
    ])
}
fn square() -> Shape {
    Shape(smallvec![
        Pos { row: 0, col: 0 },
        Pos { row: 0, col: 1 },
        Pos { row: -1, col: 0 },
        Pos { row: -1, col: 1 },
    ])
}

#[derive(Debug)]
struct Game {
    m: HashSet<Pos>,
    active: Option<Shape>,
    top: i64,
}

impl Game {
    fn new() -> Self {
        Game {
            m: Default::default(),
            active: None,
            top: -1,
        }
    }

    fn finalize(&mut self) {
        if let Some(s) = &mut self.active {
            self.top = self.top.max(s.0.iter().map(|p| p.row).max().unwrap());
            self.m.extend(&s.0);
            self.active = None;
        }
    }

    fn highest(&self) -> i64 {
        self.active
            .as_ref()
            .and_then(|s| s.0.iter().map(|p| p.row).max())
            .unwrap_or(-1)
            .max(self.top)
    }

    fn add(&mut self, s: &Shape) {
        assert!(self.active.is_none());
        let mut s = s.clone();
        while s.left_most() < 2 {
            s.apply(RIGHT);
        }

        let highest = self.highest() + 4;
        let times = highest - s.lowest();
        s.apply_n(UP, times);

        self.active = Some(s);
    }

    fn is_free(&self, p: Pos) -> bool {
        if p.col < 0 || p.col >= 7 {
            return false;
        }
        if p.row < 0 {
            return false;
        }
        !self.m.contains(&p)
    }

    fn turn(&mut self, dir: Pos) -> bool {
        if let Some(s) = &self.active {
            let mut next = s.clone();
            next.apply(dir);
            if next.0.iter().all(|p| self.is_free(*p)) {
                self.active = Some(next);
                return true;
            }
        }
        false
    }
}

fn run(wind: &[Pos], max: usize) -> Vec<i64> {
    let mut g = Game::new();

    let shapes = vec![line(), cross(), l(), line_down(), square()];

    let mut next_wind = 0;
    let mut next_shape = 0;

    g.add(&shapes[next_shape % shapes.len()]);
    next_shape += 1;

    let mut hights = vec![0];

    for _ in 1.. {
        let w = wind[next_wind % wind.len()];
        g.turn(w);
        let can_move = g.turn(DOWN);
        next_wind += 1;
        if !can_move {
            if next_shape == max {
                break;
            }
            g.finalize();
            let next_h = g.highest();
            hights.push(next_h + 1);
            g.add(&shapes[next_shape % shapes.len()]);
            next_shape += 1;
        }
    }
    hights
}

fn find_cycle(heights: &[i64]) -> (i64, i64) {
    for off in 1.. {
        let idxs: V<_> = (1..=5).map(|v| v * off).collect();
        let mut deltas: V<_> = idxs
            .windows(2)
            .map(|w| heights[w[1]] - heights[w[0]])
            .collect();
        deltas.sort_unstable();
        deltas.dedup();
        if deltas.len() == 1 {
            return (off as i64, deltas[0]);
        }
    }

    unreachable!()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let wind: Vec<Pos> = input.lines().next().unwrap().bytes().map(parse).collect();

    let s = Instant::now();

    let heights = run(&wind, 10000);
    let part1 = heights[2022];
    let (cycle_width, delta) = find_cycle(&heights);

    let target = 1000000000000i64;
    let times_full = target / cycle_width;
    let diff_times = target - (times_full * cycle_width);
    let base = times_full * delta;
    let part2 = base + heights[diff_times as usize];

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(3193, part1);
        assert_eq!(1577650429835, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }

    Ok(e)
}
