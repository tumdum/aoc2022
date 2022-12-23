use anyhow::Result;
use maplit::hashmap;
use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use smallvec::{smallvec, SmallVec};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

type V<T> = SmallVec<[T; 5]>;

#[derive(Debug, Clone, Copy)]
struct BuildRobot(u8);

const ORE: u8 = 0;
const CLAY: u8 = 1;
const OBSIDIAN: u8 = 2;
const GEODE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    robots: [u8; 4],
    resources: [u8; 4],
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(u32::from_le_bytes(self.robots));
        state.write_u32(u32::from_le_bytes(self.resources));
    }
}

impl State {
    fn new() -> Self {
        Self {
            robots: [1, 0, 0, 0],
            resources: [0, 0, 0, 0],
        }
    }

    fn current_score(&self) -> u8 {
        self.resources[GEODE as usize]
    }

    fn next_states(&self, blueprint: &Blueprint) -> V<State> {
        let mut v = smallvec![generate_resources(*self)];
        'out: for (name, costs) in blueprint.cost.iter().enumerate() {
            for (resource, cost) in costs.iter().enumerate() {
                if self.resources[resource as usize] < *cost {
                    continue 'out;
                }
            }
            let mut new_state = *self;
            for (resource, cost) in costs.iter().enumerate() {
                new_state.resources[resource as usize] -= cost;
            }
            new_state = generate_resources(new_state);
            new_state.robots[name as usize] += 1; //.entry(name.clone()).or_default() += 1;
                                                  // v.push((new_state, Some(BuildRobot(name as u8))));
            if name == GEODE as usize {
                return smallvec![new_state];
            }
            v.push(new_state);
        }
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Inner {
    state: State,
    time: u8,
}

impl Inner {
    fn new() -> Self {
        Self {
            state: State::new(),
            time: 0,
        }
    }
}

fn find_best(
    blueprint: &Blueprint,
    max_time: u8,
    start: &[Inner],
    keep_last: bool,
) -> (usize, Vec<Inner>) {
    // let mut todo: VecDeque<Reverse<Inner>> = VecDeque::new();
    let mut todo: BinaryHeap<Reverse<Inner>> = Default::default();
    let mut seen: HashMap<State, u8> = Default::default();
    for v in start {
        todo.push(Reverse(*v));
    }

    let mut all_last = vec![];
    let mut best_score = 0;

    while let Some(Reverse(inner)) = todo.pop() {
        if inner.time == max_time {
            if keep_last {
                all_last.push(inner);
            }
            if best_score < inner.state.current_score() {
                best_score = best_score.max(inner.state.current_score());
            }
            continue;
        }
        for next_state in inner.state.next_states(blueprint) {
            let next_inner = Inner {
                state: next_state,
                time: inner.time + 1,
            };
            let t = seen
                .get(&next_inner.state)
                .copied()
                .unwrap_or(u8::max_value());
            if t <= next_inner.time {
                continue;
            } else {
                seen.insert(next_inner.state, next_inner.time);
            }
            todo.push(Reverse(next_inner));
        }
    }

    (best_score as usize, all_last)
}

#[derive(Debug)]
struct Blueprint {
    id: usize,
    cost: [[u8; 4]; 4],
}

impl Blueprint {
    #[allow(unused)]
    fn dbg(&self, m: &HashMap<&str, u8>) {
        let m: HashMap<u8, &str> = m.iter().map(|(name, i)| (*i, *name)).collect();
        let cost: HashMap<&str, HashMap<&str, u8>> = self
            .cost
            .iter()
            .enumerate()
            .map(|(id, costs)| {
                (
                    *m.get(&(id as u8)).unwrap(),
                    costs
                        .iter()
                        .enumerate()
                        .map(|(id, cost)| (*m.get(&(id as u8)).unwrap(), *cost))
                        .filter(|(_, cost)| *cost > 0)
                        .collect(),
                )
            })
            .collect();
        dbg!(cost);
    }
}

fn parse(s: &str, m: &HashMap<&str, u8>) -> Blueprint {
    let s = s.replace('.', "");
    let s: Vec<_> = s.split(' ').collect();
    let id: usize = s[1].strip_suffix(':').unwrap().parse().unwrap();

    let mut cost = [[0; 4]; 4];
    cost[ORE as usize][*m.get(s[7]).unwrap() as usize] = s[6].parse().unwrap();
    cost[CLAY as usize][*m.get(s[13]).unwrap() as usize] = s[12].parse().unwrap();
    cost[OBSIDIAN as usize][*m.get(s[19]).unwrap() as usize] = s[18].parse().unwrap();
    cost[OBSIDIAN as usize][*m.get(s[22]).unwrap() as usize] = s[21].parse().unwrap();
    cost[GEODE as usize][*m.get(s[28]).unwrap() as usize] = s[27].parse().unwrap();
    cost[GEODE as usize][*m.get(s[31]).unwrap() as usize] = s[30].parse().unwrap();
    Blueprint { id, cost }
}

fn generate_resources(mut state: State) -> State {
    for (name, count) in state.robots.iter().enumerate() {
        state.resources[name] += count;
    }
    state
}

fn compute_part2(blueprint: &Blueprint) -> usize {
    let (ret, mut all_last) = find_best(blueprint, 24, &[Inner::new()], true);
    all_last.sort_unstable_by_key(|inner| Reverse(inner.state.current_score()));
    if ret > 0 {
        all_last.truncate(all_last.len() / 10);
    }
    find_best(blueprint, 32, &all_last, false).0
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let m: HashMap<&str, u8> = hashmap! {
        "clay" => CLAY,
        "obsidian" => OBSIDIAN,
        "ore" => ORE,
        "geode" => GEODE,
    }
    .into_iter()
    .collect();

    let input: Vec<Blueprint> = input.lines().map(|s| parse(s, &m)).collect();

    let s = Instant::now();

    let part1 = input
        .par_iter()
        .map(|b| b.id * find_best(b, 24, &[Inner::new()], false).0)
        .sum::<usize>();
    assert_eq!(1395, part1);
    let part2 = input
        .par_iter()
        .take(3)
        .map(compute_part2)
        .product::<usize>();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(1395, part1);
        assert_eq!(2700, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
