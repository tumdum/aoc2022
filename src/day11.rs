use anyhow::{anyhow, Error, Result};
use std::str::FromStr;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct Test {
    divisible_by: u64,
    if_true: usize,
    if_false: usize,
}

impl Test {
    fn apply(&self, wl: u64) -> usize {
        if wl % self.divisible_by == 0 {
            self.if_true
        } else {
            self.if_false
        }
    }

    fn parse(lines: &[&str]) -> Self {
        let divisible_by: u64 = lines[0]
            .strip_prefix("  Test: divisible by ")
            .unwrap()
            .parse()
            .unwrap();
        let if_true: usize = lines[1]
            .strip_prefix("    If true: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();
        let if_false: usize = lines[2]
            .strip_prefix("    If false: throw to monkey ")
            .unwrap()
            .parse()
            .unwrap();
        Self {
            divisible_by,
            if_true,
            if_false,
        }
    }
}

#[derive(Clone, Debug)]
enum Num {
    N(u64),
    Old,
}

impl Num {
    fn eval(&self, wl: u64) -> u64 {
        match self {
            Self::N(n) => *n,
            Self::Old => wl,
        }
    }
}

impl FromStr for Num {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s == "old" {
            Ok(Num::Old)
        } else {
            Ok(Num::N(s.parse().unwrap()))
        }
    }
}

#[derive(Clone, Debug)]
enum Operation {
    Times(Num),
    Add(Num),
}

impl Operation {
    fn apply(&self, wl: u64) -> u64 {
        match self {
            Self::Add(n) => n.eval(wl) + wl,
            Self::Times(n) => n.eval(wl) * wl,
        }
    }
}

impl FromStr for Operation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.strip_prefix("  Operation: new = old ").unwrap();
        let mut s = s.split(' ');
        match s.next().unwrap() {
            "*" => Ok(Operation::Times(s.next().unwrap().parse().unwrap())),
            "+" => Ok(Operation::Add(s.next().unwrap().parse().unwrap())),
            s => Err(anyhow!("unexpected '{s}'")),
        }
    }
}

#[derive(Clone, Debug)]
struct Monkey {
    items: Vec<u64>,
    operation: Operation,
    test: Test,
    inspect_count: usize,
}

impl Monkey {
    fn parse(lines: &[&str]) -> Self {
        let starting = lines[1];
        let items: Vec<u64> = starting
            .strip_prefix("  Starting items: ")
            .unwrap()
            .split(", ")
            .map(|n| n.parse().unwrap())
            .collect();
        let operation = lines[2].parse().unwrap();
        let test = Test::parse(&lines[3..]);
        Self {
            items,
            operation,
            test,
            inspect_count: 0,
        }
    }
}

fn simulate(mut monkeys: Vec<Monkey>, part: usize) -> usize {
    let modulo: u64 = if part == 1 {
        u64::max_value()
    } else {
        monkeys.iter().map(|m| m.test.divisible_by).product()
    };
    let rounds = if part == 1 { 20 } else { 10000 };
    let divisor = if part == 1 { 3 } else { 1 };
    for _ in 1..=rounds {
        for m in 0..monkeys.len() {
            monkeys[m].inspect_count += monkeys[m].items.len();
            for i in 0..monkeys[m].items.len() {
                let item = monkeys[m].items[i];
                let worry_level = monkeys[m].operation.apply(item) / divisor;
                let target_monkey = monkeys[m].test.apply(worry_level);
                monkeys[target_monkey].items.push(worry_level % modulo);
            }
            monkeys[m].items.clear();
        }
    }
    let mut inspect_counts: Vec<_> = monkeys.iter().map(|m| m.inspect_count).collect();
    inspect_counts.sort_unstable();
    inspect_counts.reverse();
    inspect_counts.into_iter().take(2).product()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();
    let monkeys: Vec<Monkey> = input.split(|l| l.is_empty()).map(Monkey::parse).collect();

    let s = Instant::now();

    let part1 = simulate(monkeys.clone(), 1);
    let part2 = simulate(monkeys, 2);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(151312, part1);
        assert_eq!(51382025916, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
