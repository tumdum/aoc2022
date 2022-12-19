use anyhow::Result;
use maplit::{btreemap, hashmap};
use rustc_hash::FxHashSet as HashSet;
use smallvec::{smallvec, SmallVec};
use std::collections::{BinaryHeap, BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::cmp::Reverse;

type V<T> = SmallVec<[T; 5]>;

#[derive(Debug, Clone)]
struct BuildRobot(u8);

type Minute = u32;
type Actions = HashMap<Minute, BuildRobot>;

const ORE: u8 = 0;
const CLAY: u8 = 1;
const OBSIDIAN: u8 = 2;
const GEODE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct State {
    robots: [u8; 4],
    resources: [u8; 4],
    // total_score: u8,
}

impl State {
    fn new() -> Self {
        Self {
            robots: [1, 0, 0, 0],
            resources: [0, 0, 0, 0],
            // total_score: 0,
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
            let mut new_state = self.clone();
            for (resource, cost) in costs.iter().enumerate() {
                new_state.resources[resource as usize] -= cost;
            }
            new_state = generate_resources(new_state);
            new_state.robots[name as usize] += 1; //.entry(name.clone()).or_default() += 1;
                                                  // v.push((new_state, Some(BuildRobot(name as u8))));
            v.push(new_state);
        }
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Inner {
    state: State,
    time: u8,
    // total_score: u8,
}

impl Inner {
    fn new() -> Self {
        Self {
            state: State::new(),
            time: 0,
        }
    }
}

fn find_best(blueprint: &Blueprint, max_time: u8, start: &[Inner], keep_last: bool, m: &HashMap<&str, u8>) -> (usize, Vec<Inner>) {
    let total_score_for_new = compute_total_score(1, max_time as u32, &State::new());
    let mut rej = 0;
    let mut todo: VecDeque<Reverse<Inner>> = VecDeque::new();
    let mut todo: BinaryHeap<Reverse<Inner>> = Default::default();
    let mut seen: HashSet<Inner> = Default::default();
    for v in start {
        todo.push(Reverse(*v));
        seen.insert(*v);
    }

    let mut all_last = vec![];

    let mut best_score = 0;
    let mut best = State::new();
    let mut rejected = 0;

    while let Some(Reverse(inner)) = todo.pop() {
        if inner.time == max_time {
            if seen.len() > 200000000 {
                seen.clear();
            }
            if keep_last {
                all_last.push(inner);
            }
            if rej % 10000000 == 0 {
                println!(
                    "{}: todo {} best {} current {} rej {} seen {} all_last {}",
                    blueprint.id,
                    todo.len(),
                    best_score,
                    inner.state.current_score(),
                    rej,
                    seen.len(),
                    all_last.len(),
                );
            }
            if best_score < inner.state.current_score() {
                best_score = best_score.max(inner.state.current_score());
                best = inner.state.clone();
            }
            continue;
        }
        for next_state in inner.state.next_states(&blueprint) {
            let next_inner = Inner {
                state: next_state,
                time: inner.time + 1,
            };
            if seen.contains(&next_inner) {
                rej += 1;
                continue;
            }
            todo.push(Reverse(next_inner));
            seen.insert(next_inner);
        }
    }

    blueprint.dbg(&m);
    (dbg!(best_score) as usize, all_last)
}

#[derive(Debug)]
struct Blueprint {
    id: usize,
    // cost: HashMap<u8, HashMap<u8, u8>>,
    cost: [[u8; 4]; 4],
}

impl Blueprint {
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
        println!("Blueprint #{}: {cost:?}", self.id);
    }
}

fn parse(s: &str, m: &HashMap<&str, u8>) -> Blueprint {
    let s = s.replace(".", "");
    dbg!(&s);
    let s: Vec<_> = s.split(' ').collect();
    let id: usize = s[1].strip_suffix(':').unwrap().parse().unwrap();

    let costs = vec![
        s[3], s[6], s[9], s[12], s[15], s[18], s[19], s[21], s[22], s[24], s[27], s[30], s[31],
    ];

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

fn run(actions: &Actions, turns: u32, blueprint: &Blueprint) -> u8 {
    let mut state = State::new();

    for minute in 1u32..=turns {
        state = generate_resources(state);
        if let Some(BuildRobot(kind)) = actions.get(&minute) {
            state.robots[*kind as usize] += 1;
            for (name, count) in blueprint.cost[*kind as usize].iter().enumerate() {
                state.resources[name] -= count;
            }
        }
    }
    state.resources[GEODE as usize]
}

fn compute_total_score(current_turn: u32, last_turn: u32, state: &State) -> u8 {
    let mut state = *state;

    for minute in current_turn..=last_turn {
        state = generate_resources(state);
    }
    let ret = state.resources[GEODE as usize];
    ret
}

fn compute_part2(blueprint: &Blueprint, m: &HashMap<&str, u8> ) -> usize {
    let (ret, mut all_last) = find_best(blueprint, 24, &[Inner::new()], true, m);
    println!("{}: {} {}", blueprint.id, ret, all_last.len());
    all_last.sort_unstable_by_key(|inner| Reverse(inner.state.current_score()));
    if ret > 0 {
        all_last.truncate(all_last.len()/10);
    }
    find_best(blueprint, 32, &all_last, false, m).0
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let m: HashMap<&str, u8> = hashmap! {
        "clay" => CLAY,
        "obsidian" => OBSIDIAN,
        "ore" => ORE,
        "geode" => GEODE,
    };

    let input: Vec<Blueprint> = input.lines().map(|s| parse(s, &m)).collect();
    dbg!(&input);

    let s = Instant::now();

    /*
    let test_actions = hashmap! {
        3 => BuildRobot("clay".to_owned()),
        5 => BuildRobot("clay".to_owned()),
        7 => BuildRobot("clay".to_owned()),
        11 => BuildRobot("obsidian".to_owned()),
        12 => BuildRobot("clay".to_owned()),
        15 => BuildRobot("obsidian".to_owned()),
        18 => BuildRobot("geode".to_owned()),
        21 => BuildRobot("geode".to_owned()),

    };
    debug_assert_eq!(9, run(&test_actions, 24, &input[0]));
    */

    /*
    let (ret, mut all_last) = find_best(&input[0], 24, &[Inner::new()], true, &m);
    use std::cmp::Reverse;
    all_last.sort_unstable_by_key(|inner| Reverse(inner.state.current_score()));
    all_last.truncate(all_last.len()/1000);
    assert_eq!(9, ret);
    assert_eq!(56, find_best(&input[0], 32, &all_last, false, &m).0);
    */
    /*
    assert_eq!(56, compute_part2(&input[0], &m));
    assert_eq!(62, compute_part2(&input[1], &m));
    */

    // assert_eq!(12, find_best(&input[1], 24, &[Inner::new()], true, &m));
    // assert_eq!(56, find_best(&input[0], 32, &m));
    // assert_eq!(62, find_best(&input[1], 32, &m));
    // dbg!(input.iter().map(|b| b.id * find_best(b, 24, &m)).sum::<usize>());
    //
    use rayon::prelude::*;
    dbg!(input.iter().take(3).rev().map(|b| compute_part2(b, &m)).product::<usize>());

    let e = s.elapsed();

    if verify_expected {
        // assert_eq!(7917, part1);
        // assert_eq!(2585, part2);
    }
    if output {
        // println!("\t{}", part1);
        // println!("\t{}", part2);
    }
    Ok(e)
}
