use anyhow::Result;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct P {
    row: i32,
    col: i32,
}

impl P {
    const fn add(self, other: Self) -> Self {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }

    fn adjacent_iter(self, m: &'_ HashSet<P>) -> impl Iterator<Item = P> + '_ + Clone {
        NEIGHBOURS
            .into_iter()
            .filter(move |dir| m.contains(&self.add(*dir)))
    }

    fn name(self) -> String {
        if self == N {
            return "N".to_owned();
        }
        if self == S {
            return "S".to_owned();
        }
        if self == E {
            return "E".to_owned();
        }
        if self == W {
            return "W".to_owned();
        }

        if self == NE {
            return "NE".to_owned();
        }
        if self == SE {
            return "SE".to_owned();
        }
        if self == NW {
            return "NW".to_owned();
        }
        if self == SW {
            return "SW".to_owned();
        }

        unreachable!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Rule {
    free_dirs: [P; 3],
    move_dir: P,
}

impl Rule {
    fn passes(&self, mut non_empty_adjacent: impl Iterator<Item = P>) -> bool {
        non_empty_adjacent.all(|dir| !self.free_dirs.contains(&dir))
    }

    #[allow(unused)]
    fn print(&self) -> String {
        format!(
            "Rule{{ free_dirs: {:?}, move_dir: {} }}",
            self.free_dirs.iter().map(|p| p.name()).collect_vec(),
            self.move_dir.name()
        )
    }
}

const RULES: [Rule; 4] = [
    Rule {
        free_dirs: [N, NE, NW],
        move_dir: N,
    },
    Rule {
        free_dirs: [S, SE, SW],
        move_dir: S,
    },
    Rule {
        free_dirs: [W, NW, SW],
        move_dir: W,
    },
    Rule {
        free_dirs: [E, NE, SE],
        move_dir: E,
    },
];

const UP: P = P { row: -1, col: 0 };
const DOWN: P = P { row: 1, col: 0 };
const LEFT: P = P { row: 0, col: -1 };
const RIGHT: P = P { row: 0, col: 1 };

const N: P = UP;
const S: P = DOWN;
const W: P = LEFT;
const E: P = RIGHT;

const NE: P = N.add(E);
const NW: P = N.add(W);
const SE: P = S.add(E);
const SW: P = S.add(W);

const NEIGHBOURS: [P; 8] = [N, S, W, E, NE, NW, SE, SW];

fn next_step(m: &HashSet<P>, rules: &VecDeque<Rule>) -> HashSet<P> {
    let mut ret = Vec::with_capacity(m.len());
    let mut proposed: HashMap<P, Vec<P>> = Default::default();
    for elf in m {
        let mut adjacent = elf.adjacent_iter(m).peekable();
        if adjacent.peek().is_none() {
            ret.push(*elf);
            continue;
        }
        let mut proposed_already = false;
        for rule in rules {
            if rule.passes(adjacent.clone()) {
                proposed
                    .entry(elf.add(rule.move_dir))
                    .or_default()
                    .push(*elf);
                proposed_already = true;
                break;
            }
        }
        if !proposed_already {
            ret.push(*elf);
        }
    }
    for (target, candidates) in proposed {
        debug_assert!(candidates.len() >= 1);
        if candidates.len() == 1 {
            ret.push(target);
        } else {
            for cand in candidates {
                ret.push(cand);
            }
        }
    }
    ret.into_iter().collect()
}

#[allow(unused)]
fn print(m: &HashSet<P>) {
    let (minx, maxx) = m.iter().map(|p| p.col).minmax().into_option().unwrap();
    let (miny, maxy) = m.iter().map(|p| p.row).minmax().into_option().unwrap();
    for row in miny..=maxy {
        for col in minx..=maxx {
            if m.contains(&P { row, col }) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

fn count_empty(m: &HashSet<P>) -> usize {
    let mut ret = 0;
    let (minx, maxx) = m.iter().map(|p| p.col).minmax().into_option().unwrap();
    let (miny, maxy) = m.iter().map(|p| p.row).minmax().into_option().unwrap();
    for row in miny..=maxy {
        for col in minx..=maxx {
            if !m.contains(&P { row, col }) {
                ret += 1;
            }
        }
    }
    ret
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();
    let mut map: HashSet<P> = Default::default();
    for row in 0..input.len() {
        for col in 0..input[row].len() {
            let b = input[row][col];
            if b == b'#' {
                map.insert(P {
                    row: row as i32,
                    col: col as i32,
                });
            }
        }
    }
    let mut rules: VecDeque<Rule> = RULES.into_iter().collect();
    let s = Instant::now();
    let mut current = map.clone();
    let mut part1 = 0;
    let mut part2 = 0;
    for turn in 1.. {
        let next = next_step(&current, &rules);
        if next == current {
            part2 = turn;
            break;
        }
        current = next;
        let first = rules.pop_front();
        rules.push_back(first.unwrap());
        if turn == 10 {
            part1 = count_empty(&current);
        }
    }

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(3788, part1);
        assert_eq!(921, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
