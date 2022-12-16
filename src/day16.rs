use anyhow::Result;
use itertools::{iproduct, Itertools};
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::time::{Duration, Instant};

type G = HashMap<String, (usize, Vec<(String, usize)>)>;
type V<T> = SmallVec<[T; 8]>;

fn parse(s: &str) -> (String, (usize, Vec<(String, usize)>)) {
    let s = s.split(' ').collect_vec();
    let from = s[1];
    let rate: usize = s[4]
        .strip_prefix("rate=")
        .unwrap()
        .strip_suffix(";")
        .unwrap()
        .parse()
        .unwrap();
    let to = s[9..]
        .iter()
        .map(|s| (s.strip_suffix(',').unwrap_or(s).to_owned(), 1))
        .collect_vec();
    (from.to_owned(), (rate, to))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Action {
    Open(String),
    MoveTo(String),
}

use Action::*;

fn total_score(
    actions: &[Action],
    start_node: &str,
    g: &HashMap<String, (usize, Vec<(String, usize)>)>,
    max_minutes: usize,
) -> (usize, String, usize) {
    let mut s = 0;
    let mut minute = 1;
    let mut next_a = 0;
    let mut current = start_node.to_owned();
    let mut turns = 0;
    loop {
        if next_a >= actions.len() {
            break;
        }
        match &actions[next_a] {
            MoveTo(target) => {
                let t = g
                    .get(&current)
                    .unwrap()
                    .1
                    .iter()
                    .find(|(name, _)| name == target)
                    .map(|(_, cost)| cost)
                    .unwrap();
                turns += t;
                current = target.to_owned();
            }
            Open(v) => {
                turns += 1;
                assert_eq!(&current, v);
                let rate = g.get(&*current).unwrap().0;
                s += (max_minutes - minute) * rate
            }
        }

        minute += 1;
        next_a += 1;
    }
    (s, current, turns)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    actions: V<Action>,
    current_node: String,
    score: usize,
    turn: usize,
}
impl State {
    fn new(
        actions: &[Action],
        start_node: &str,
        g: &HashMap<String, (usize, Vec<(String, usize)>)>,
        max_minutes: usize,
    ) -> Self {
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
        // other.0.score.cmp(&self.0.score)
        let ret = self.0.score.cmp(&other.0.score);
        if ret == Ordering::Equal {
            other.0.actions.len().cmp(&self.0.actions.len())
        } else {
            ret
        }
    }
}

impl PartialOrd for Tmp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_path(g: &G, from: &str, to: &str) -> Vec<Action> {
    let mut best: HashMap<String, (usize, String)> = HashMap::default();
    best.insert(from.to_owned(), (0, from.to_owned()));
    let mut todo: VecDeque<(String, usize)> = VecDeque::new();
    todo.push_back((from.to_owned(), 0));

    while let Some((node, cost)) = todo.pop_front() {
        if node == to {
            break;
        }
        for (neighbour, n_cost) in &g.get(&node).unwrap().1 {
            assert_eq!(1, *n_cost);
            let new_cost = cost + n_cost;
            if best
                .get(&*neighbour)
                .map(|(cost, _)| *cost)
                .unwrap_or(usize::max_value())
                > new_cost
            {
                todo.push_back((neighbour.to_owned(), new_cost));
                best.insert(neighbour.to_owned(), (new_cost, node.clone()));
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
        path.push(cur.clone());
    }
    path.push(cur);
    assert_eq!(Some(from.to_owned()), path.pop());
    path.reverse();
    let mut actions: Vec<Action> = path.into_iter().map(|node| MoveTo(node)).collect();
    actions.push(Open(to.to_owned()));

    actions
}

fn part1(g: &G) -> usize {
    #[derive(Debug, Clone)]
    struct World {
        activated: HashSet<String>,
        actions: Vec<Action>,
        current_node: String,
    }
    let all_non_zero_valves: HashSet<String> = g
        .iter()
        .map(|(name, (rate, _))| (name, rate))
        .filter(|(_, rate)| **rate != 0)
        .map(|(name, _)| name.clone())
        .collect();
    let start_state = World {
        activated: HashSet::default(),
        actions: vec![],
        current_node: "AA".to_owned(),
    };
    let mut todo: VecDeque<(World, usize)> = VecDeque::new();
    let mut best: HashMap<Vec<String>, usize> = HashMap::default();
    todo.push_back((start_state.clone(), 0));
    let mut best_score = 0;
    while let Some((world, score)) = todo.pop_front() {
        if best_score < score {
            best_score = best_score.max(score);
        }
        let targets = all_non_zero_valves.difference(&world.activated);
        for target in targets {
            let new_actions = find_path(g, &world.current_node, &target);
            let mut actions = world.actions.clone();
            actions.extend(new_actions);
            if actions.len() > 30 {
                continue;
            }
            let mut new_activated = world.activated.clone();
            new_activated.insert(target.clone());

            let new_world = World {
                current_node: target.clone(),
                actions,
                activated: new_activated,
            };
            let state = State::new(&new_world.actions, "AA", g, 30);
            let mut new_activated_vec = new_world.activated.iter().cloned().collect_vec();
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

fn part2(g: &G) -> usize {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct World {
        activated: HashSet<String>,
        actions: V<Action>,
        current_node: String,
    }
    let all_non_zero_valves: HashSet<String> = g
        .iter()
        .map(|(name, (rate, _))| (name, rate))
        .filter(|(_, rate)| **rate != 0)
        .map(|(name, _)| name.clone())
        .collect();
    let mut all_actions: HashMap<(String, String), Vec<Action>> = Default::default();
    for from in all_non_zero_valves
        .iter()
        .chain(std::iter::once(&"AA".to_owned()))
    {
        for to in &all_non_zero_valves {
            all_actions.insert((from.to_owned(), to.to_owned()), find_path(g, from, to));
        }
    }
    println!("{all_non_zero_valves:?} {}", all_non_zero_valves.len());
    let start_state = World {
        activated: HashSet::default(),
        actions: smallvec![],
        current_node: "AA".to_owned(),
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
    let mut best: HashMap<Vec<String>, usize> = HashMap::default();
    todo.push(Order(start_state.clone(), start_state.clone(), 0, 0));
    let mut best_score = 0;
    let mut c = 0;
    let mut rejected = 0;
    while let Some(Order(my_world, elephant_world, my_score, elephant_score)) = todo.pop() {
        c += 1;
        if c % 10000 == 0 {
            println!(
                "considering {}, considered {} rejected {} best {} ({:?},{:?}) ({},{}) {}",
                todo.len(),
                c,
                rejected,
                best_score,
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
        let my_targets: HashSet<String> = all_non_zero_valves
            .difference(&my_world.activated)
            .cloned()
            .collect();
        let targets = my_targets.difference(&elephant_world.activated);
        fn make_new_world(
            g: &G,
            world: &World,
            target: &str,
            actions: &HashMap<(String, String), Vec<Action>>,
        ) -> Option<(World, usize)> {
            let new_actions = actions
                .get(&(world.current_node.clone(), target.to_owned()))
                .unwrap()
                .clone();
            let mut actions = world.actions.clone();
            actions.extend(new_actions);
            if actions.len() > 26 {
                return None;
            }
            let mut new_activated = world.activated.clone();
            new_activated.insert(target.to_owned());

            let new_world = World {
                current_node: target.to_owned(),
                actions,
                activated: new_activated,
            };
            let (score, _, _) = total_score(&new_world.actions, "AA", g, 26);
            Some((new_world, score))
        }
        for (my_target, elephant_target) in iproduct!(targets.clone(), targets) {
            if my_target == elephant_target {
                continue;
            }

            let my_new_world = make_new_world(g, &my_world, &my_target, &all_actions);
            let elephant_new_world =
                make_new_world(g, &elephant_world, elephant_target, &all_actions);
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
            let all_new_activated: Vec<String> = my_new_activated
                .chain(e_new_activated)
                .sorted()
                .cloned()
                .collect();
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
    let input: HashMap<String, (usize, Vec<(String, usize)>)> = input.lines().map(parse).collect();

    let s = Instant::now();

    // Works but is tragically slow
    let part1 = part1(&input);
    assert_eq!(1820, part1);
    let part2 = part2(&input);

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
