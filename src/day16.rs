use crate::U8Set;
use anyhow::Result;
use itertools::{iproduct, Itertools};
use rustc_hash::FxHashMap as HashMap;
use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::time::{Duration, Instant};

type G = Vec<(i8, V<u8>)>;
type V<T> = SmallVec<[T; 14]>;
type V2<T> = SmallVec<[T; 30]>;

fn parse(s: &str) -> (String, (i8, V<String>)) {
    let s = s.split(' ').collect_vec();
    let from = s[1];
    let rate: i8 = s[4]
        .strip_prefix("rate=")
        .unwrap()
        .strip_suffix(';')
        .unwrap()
        .parse()
        .unwrap();
    let to = s[9..]
        .iter()
        .map(|s| s.strip_suffix(',').unwrap_or(s).to_owned())
        .collect();
    (from.to_owned(), (rate, to))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Open(u8),
    MoveTo(u8),
}

use Action::*;

fn total_score(
    actions: &[Action],
    start_node: u8,
    g: &G,
    max_minutes: usize,
) -> (usize, u8, usize) {
    let mut s = 0;
    let mut current = start_node;
    let mut turns = 0;
    for (minute, action) in actions.iter().enumerate().map(|(m, a)| (m + 1, a)) {
        match action {
            MoveTo(target) => {
                turns += 1;
                current = target.to_owned();
            }
            Open(v) => {
                turns += 1;
                debug_assert_eq!(&current, v);
                let rate = g[current as usize].0;
                s += (max_minutes - minute) * rate as usize
            }
        }
    }
    (s, current, turns)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    actions: V2<Action>,
    current_node: u8,
    score: usize,
    turn: usize,
}
impl State {
    fn new(actions: &[Action], start_node: u8, g: &G, max_minutes: usize) -> Self {
        let (score, current_node, turns) = total_score(actions, start_node, g, max_minutes);
        Self {
            actions: actions.into(),
            current_node,
            score,
            turn: turns,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Tmp(State, Action);

impl Ord for Tmp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.score.cmp(&other.0.score)
    }
}

impl PartialOrd for Tmp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_path(g: &G, from: u8, to: u8) -> V2<Action> {
    let mut best: HashMap<u8, (usize, u8)> = HashMap::default();
    best.insert(from.to_owned(), (0, from.to_owned()));
    let mut todo: VecDeque<(u8, usize)> = VecDeque::new();
    todo.push_back((from.to_owned(), 0));

    while let Some((node, cost)) = todo.pop_front() {
        if node == to {
            break;
        }
        for neighbour in &g[node as usize].1 {
            let new_cost = cost + 1;
            if best
                .get(neighbour)
                .map(|(cost, _)| *cost)
                .unwrap_or(usize::max_value())
                > new_cost
            {
                todo.push_back((neighbour.to_owned(), new_cost));
                best.insert(neighbour.to_owned(), (new_cost, node));
            }
        }
    }

    let mut path = vec![to.to_owned()];
    let mut cur = to.to_owned();
    while let Some((_, prev)) = best.get(&cur) {
        cur = prev.to_owned();
        if cur == from {
            break;
        }
        path.push(cur);
    }
    path.push(cur);
    assert_eq!(Some(from.to_owned()), path.pop());
    path.reverse();
    let mut actions: V2<Action> = path.into_iter().map(MoveTo).collect();
    actions.push(Open(to));

    actions
}

fn part1(g: &G, start_node: u8) -> usize {
    let mut all_non_zero_valves: U8Set = g
        .iter()
        .enumerate()
        .map(|(name, (rate, _))| (name, rate))
        .filter(|(_, rate)| **rate != 0)
        .map(|(name, _)| name as u8)
        .collect();
    all_non_zero_valves.max = 50;
    let start_state = World {
        activated: Default::default(),
        actions: smallvec![],
        current_node: start_node,
    };
    let mut todo: VecDeque<(World, usize)> = VecDeque::new();
    let mut best: HashMap<V<u8>, usize> = HashMap::default();
    todo.push_back((start_state, 0));
    let mut best_score = 0;
    while let Some((world, score)) = todo.pop_front() {
        if best_score < score {
            best_score = best_score.max(score);
        }
        let targets = all_non_zero_valves.difference(&world.activated);
        for target in targets.iter() {
            let new_actions = find_path(g, world.current_node, target);
            let mut actions = world.actions.clone();
            actions.extend(new_actions);
            if actions.len() > 30 {
                continue;
            }
            let mut new_activated = world.activated;
            new_activated.insert(target);

            let new_world = World {
                current_node: target,
                actions,
                activated: new_activated,
            };
            let state = State::new(&new_world.actions, start_node, g, 30);
            let mut new_activated_vec: V<u8> = new_world.activated.iter().collect();
            new_activated_vec.sort_unstable();
            let old_score = best.get(&new_activated_vec).copied().unwrap_or(0);
            if old_score > state.score {
            } else {
                best.insert(new_activated_vec, state.score);
            }
            todo.push_back((new_world, state.score));
        }
    }

    best_score
}

fn make_next_world(
    g: &G,
    world: &World,
    target: u8,
    actions: &[Vec<V2<Action>>],
    start_node: u8,
) -> Option<(World, usize)> {
    let mut actions_new: V2<Action> = world.actions.clone();
    actions_new.extend_from_slice(&actions[world.current_node as usize][target as usize]);

    if actions_new.len() > 26 {
        return None;
    }

    debug_assert!(!actions_new.spilled(), "{}", actions.len());

    let mut activated = world.activated;
    activated.insert(target);

    let new_world = World {
        current_node: target.to_owned(),
        actions: actions_new,
        activated,
    };
    let (score, _, _) = total_score(&new_world.actions, start_node, g, 26);
    Some((new_world, score))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct World {
    activated: U8Set,
    actions: V2<Action>,
    current_node: u8,
}

fn part2(g: &G, start_node: u8) -> usize {
    let mut all_non_zero_valves: U8Set = g
        .iter()
        .enumerate()
        .map(|(name, (rate, _))| (name, rate))
        .filter(|(_, rate)| **rate != 0)
        .map(|(name, _)| name as u8)
        .collect();
    all_non_zero_valves.max = 50;
    let l = 1 + all_non_zero_valves.iter().max().unwrap() as usize;
    let mut all_actions = vec![vec![V2::new(); l]; l];
    for from in all_non_zero_valves
        .iter()
        .chain(std::iter::once(start_node))
    {
        for to in all_non_zero_valves.iter() {
            all_actions[from as usize][to as usize] = find_path(g, from, to);
        }
    }
    let start_state = World {
        activated: U8Set::new(50),
        actions: smallvec![],
        current_node: start_node,
    };
    #[derive(Debug, PartialEq, Eq)]
    struct Order(World, World, usize, usize);
    impl Ord for Order {
        fn cmp(&self, other: &Self) -> Ordering {
            let ret = (self.2 + self.3).cmp(&(other.2 + other.3));
            if ret == Ordering::Equal {
                (other.0.actions.len() + other.1.actions.len())
                    .cmp(&(self.0.actions.len() + self.1.actions.len()))
            } else {
                ret
            }
        }
    }
    impl PartialOrd for Order {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut todo: BinaryHeap<Order> = Default::default();
    let mut best: HashMap<V<u8>, usize> = HashMap::default();
    todo.push(Order(start_state.clone(), start_state, 0, 0));
    let mut best_score = 0;
    let mut c = 0;
    let mut rejected = 0;
    let mut time = Instant::now();
    while let Some(Order(my_world, elephant_world, my_score, elephant_score)) = todo.pop() {
        c += 1;
        if c % 1000000 == 0 {
            let elapsed = time.elapsed();
            time = Instant::now();
            eprintln!(
                "[{elapsed:?}] considering {}, considered {} rejected {} best {} best cache {}; ({:?},{:?}) ({},{}) {}",
                todo.len(),
                c,
                rejected,
                best_score,
                best.len(),
                my_world.activated.len(),
                elephant_world.activated.len(),
                my_world.actions.len(),
                elephant_world.actions.len(),
                my_score + elephant_score
            );
        }
        if best_score < (my_score + elephant_score) {
            best_score = my_score + elephant_score;
        }
        let mut my_targets: U8Set = all_non_zero_valves.difference(&my_world.activated);
        my_targets.max = 50;
        let mut targets: U8Set = my_targets.difference(&elephant_world.activated);
        targets.max = 50;
        for (my_target, elephant_target) in iproduct!(targets.iter_clone(), targets.iter_clone()) {
            if my_target == elephant_target {
                continue;
            }

            let my_new_world = make_next_world(g, &my_world, my_target, &all_actions, start_node);
            let elephant_new_world = make_next_world(
                g,
                &elephant_world,
                elephant_target,
                &all_actions,
                start_node,
            );
            if my_new_world.is_none() && elephant_new_world.is_none() {
                continue;
            }
            let my_score = my_new_world
                .as_ref()
                .map(|(_, score)| *score)
                .unwrap_or(my_score);
            let elephant_score = elephant_new_world
                .as_ref()
                .map(|(_, score)| *score)
                .unwrap_or(elephant_score);
            let my_new_world = my_new_world.unwrap_or_else(|| (my_world.clone(), my_score));
            let elephant_new_world =
                elephant_new_world.unwrap_or_else(|| (elephant_world.clone(), elephant_score));

            let my_new_activated = my_new_world.0.activated.iter();
            let e_new_activated = elephant_new_world.0.activated.iter();
            let mut all_new_activated: V<u8> = my_new_activated.chain(e_new_activated).collect();
            all_new_activated.sort_unstable();
            assert!(!all_new_activated.spilled(), "{}", all_new_activated.len());
            let best_score_for_all = best.get(&all_new_activated).copied().unwrap_or(0);
            let new_total_score = my_score + elephant_score;
            if new_total_score >= best_score_for_all {
                if new_total_score > best_score_for_all {
                    best.insert(all_new_activated, new_total_score);
                }
                todo.push(Order(
                    my_new_world.0,
                    elephant_new_world.0,
                    my_score,
                    elephant_score,
                ));
            } else {
                rejected += 1;
            }
        }
    }

    best_score
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: HashMap<String, (i8, V<String>)> = input.lines().map(parse).collect();
    let m: HashMap<String, u8> = input
        .keys()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u8))
        .collect();
    let input: HashMap<u8, (i8, V<u8>)> = input
        .iter()
        .map(|(k, (rate, out))| {
            let k = *m.get(k).unwrap();
            let out = out.iter().map(|s| *m.get(s).unwrap()).collect();
            (k, (*rate, out))
        })
        .collect();

    let mut g: Vec<(i8, V<u8>)> = vec![];
    for i in input.keys().sorted() {
        g.push(input.get(i).unwrap().clone());
        assert_eq!((i + 1) as usize, g.len());
    }

    let s = Instant::now();

    // Works but is tragically slow
    let part1 = part1(&g, *m.get("AA").unwrap());
    assert_eq!(1820, part1);
    let part2 = part2(&g, *m.get("AA").unwrap());

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(1820, part1);
        assert_eq!(2602, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
