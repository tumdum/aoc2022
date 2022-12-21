use anyhow::{Error, Result};
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use smallvec::{SmallVec};

type V<T> = SmallVec<[T; 10]>;
type State = HashMap<Node, Expr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node(u32);

impl Node {
    fn new(s: &str) -> Self {
        Self(u32::from_le_bytes(s.as_bytes().try_into().unwrap()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Add,
    Sub,
    Mul,
    Div,
    Eql,
}
use Kind::*;

impl Kind {
    fn eval(self, l: i64, r: i64) -> i64 {
        match self {
            Add => l + r,
            Sub => l - r,
            Mul => l * r,
            Div => l / r,
            Eql => (l == r) as i64,
        }
    }
}

impl FromStr for Kind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Kind> {
        match s {
            "+" => Ok(Add),
            "-" => Ok(Sub),
            "*" => Ok(Mul),
            "/" => Ok(Div),
            "=" => Ok(Eql),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expr {
    Var,
    Const(i64),
    Op(Node, Kind, Node),
}

impl Expr {
    fn make_op(&mut self, kind: Kind) {
        if let Op(l, _, r) = self {
            *self = Op(l.to_owned(), kind, r.to_owned())
        }
    }
}

use Expr::*;

fn parse(s: &str) -> (Node, Expr) {
    let s = s.split(' ').collect_vec();
    let name = s[0].strip_suffix(':').unwrap();
    let expr = if s.len() == 2 {
        Const(s[1].parse().unwrap())
    } else {
        let l = Node::new(s[1]);
        let op = s[2].parse().unwrap();
        let r = Node::new(s[3]);
        Op(l, op, r)
    };
    (Node::new(name), expr)
}

fn eval(mut state: State) -> State {
    let root = Node::new("root");
    while let Some(Op(_, _, _)) = state.get(&root) {
        let new_vals : V<(Node, i64)> = state
            .iter()
            .filter_map(|(n, v)| {
                if let Op(l, op, r) = v {
                    match (state.get(l), state.get(r)) {
                        (Some(Const(a)), Some(Const(b))) => Some((*n, op.eval(*a, *b))),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        if new_vals.is_empty() {
            break;
        }
        println!("{}", new_vals.len());
        for (name, v) in new_vals {
            state.insert(name, Const(v));
        }
    }
    state
}

fn simplify(mut state: State) -> State {
    state = eval(state);
    let root = Node::new("root");

    loop {
        if let Op(l, Eql, v_name) = *state.get(&root).unwrap() {
            if let Const(v) = *state.get(&v_name).unwrap() {
                let l = *state.get(&l).unwrap();
                match l {
                    Op(l, Div, r) => {
                        if let Const(r) = *state.get(&r).unwrap() {
                            let v = Const(v * r);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(l.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                    }
                    Op(l, Sub, r) => {
                        if let Const(r) = state.get(&r).unwrap() {
                            let v = Const(v + r);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(l.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                        if let Const(l) = state.get(&l).unwrap() {
                            let v = Const(l - v);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(r.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                    }
                    Op(l, Add, r) => {
                        if let Const(l) = state.get(&l).unwrap() {
                            let v = Const(v - l);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(r.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                        if let Const(r) = state.get(&r).unwrap() {
                            let v = Const(v - r);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(l.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                    }
                    Op(l, Mul, r) => {
                        if let Const(l) = state.get(&l).unwrap() {
                            let v = Const(v / l);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(r.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                        if let Const(r) = state.get(&r).unwrap() {
                            let v = Const(v / r);
                            state.insert(v_name.to_owned(), v);
                            state.insert(
                                Node::new("root"),
                                Op(l.to_owned(), Eql, v_name.to_owned()),
                            );
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }
    state
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: State = input.lines().map(parse).collect();

    let s = Instant::now();

    let state = eval(input.clone());
    let part1 = match state.get(&Node::new("root")) {
        Some(Const(n)) => *n,
        _ => unreachable!(),
    };

    let mut state = input;
    state.insert(Node::new("humn"), Var);
    state.get_mut(&Node::new("root")).unwrap().make_op(Eql);
    state = simplify(state);

    let part2 = match state.get(&Node::new("root")) {
        Some(Op(l, Eql, r)) => match (state.get(l), state.get(r)) {
            (Some(Const(n)), _) => *n,
            (_, Some(Const(n))) => *n,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(21208142603224, part1);
        assert_eq!(3882224466191, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
