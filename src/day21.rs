use anyhow::{Error, Result};
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, Instant};

type V<T> = SmallVec<[T; 10]>;
type State = Vec<Expr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Node(usize);

static ROOT: AtomicUsize = AtomicUsize::new(0);
static HUMN: AtomicUsize = AtomicUsize::new(0);

impl Node {
    fn new(s: &str) -> Self {
        Self(u32::from_le_bytes(s.as_bytes().try_into().unwrap()) as usize)
    }
}

impl From<&AtomicUsize> for Node {
    fn from(a: &AtomicUsize) -> Node {
        Node(a.load(SeqCst))
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
    let root = Node::from(&ROOT);
    while let Op(_, _, _) = state[root.0] {
        let new_vals: V<(Node, i64)> = state
            .iter()
            .enumerate()
            .filter_map(|(n, v)| {
                if let Op(l, op, r) = v {
                    match (state[l.0], state[r.0]) {
                        (Const(a), Const(b)) => Some((Node(n), op.eval(a, b))),
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
        for (name, v) in new_vals {
            state[name.0] = Const(v);
        }
    }
    state
}

fn simplify(mut state: State) -> State {
    state = eval(state);
    let root = Node::from(&ROOT);

    loop {
        if let Op(l, Eql, v_name) = state[root.0] {
            if let Const(v) = state[v_name.0] {
                let l = state[l.0];
                match l {
                    Op(l, Div, r) => {
                        if let Const(r) = state[r.0] {
                            let v = Const(v * r);
                            state[v_name.0] = v;
                            state[root.0] = Op(l.to_owned(), Eql, v_name.to_owned());
                        }
                    }
                    Op(l, Sub, r) => {
                        if let Const(r) = state[r.0] {
                            let v = Const(v + r);
                            state[v_name.0] = v;
                            state[root.0] = Op(l.to_owned(), Eql, v_name.to_owned());
                        }
                        if let Const(l) = state[l.0] {
                            let v = Const(l - v);
                            state[v_name.0] = v;
                            state[root.0] = Op(r.to_owned(), Eql, v_name.to_owned());
                        }
                    }
                    Op(l, Add, r) => {
                        if let Const(l) = state[l.0] {
                            let v = Const(v - l);
                            state[v_name.0] = v;
                            state[root.0] = Op(r.to_owned(), Eql, v_name.to_owned());
                        }
                        if let Const(r) = state[r.0] {
                            let v = Const(v - r);
                            state[v_name.0] = v;
                            state[root.0] = Op(l.to_owned(), Eql, v_name.to_owned());
                        }
                    }
                    Op(l, Mul, r) => {
                        if let Const(l) = state[l.0] {
                            let v = Const(v / l);
                            state[v_name.0] = v;
                            state[root.0] = Op(r.to_owned(), Eql, v_name.to_owned());
                        }
                        if let Const(r) = state[r.0] {
                            let v = Const(v / r);
                            state[v_name.0] = v;
                            state[root.0] = Op(l.to_owned(), Eql, v_name.to_owned());
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

fn fix(e: Expr, m: &HashMap<Node, usize>) -> Expr {
    match e {
        Var => Var,
        Const(n) => Const(n),
        Op(l, op, r) => Op(Node(*m.get(&l).unwrap()), op, Node(*m.get(&r).unwrap())),
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<(Node, Expr)> = input.lines().map(parse).collect();
    let m: HashMap<Node, usize> = input
        .iter()
        .enumerate()
        .map(|(i, (n, _))| (*n, i))
        .collect();
    let input: Vec<Expr> = input.into_iter().map(|(_, e)| fix(e, &m)).collect();

    let s = Instant::now();
    ROOT.store(m.get(&Node::new("root")).copied().unwrap(), SeqCst);
    HUMN.store(m.get(&Node::new("humn")).copied().unwrap(), SeqCst);

    let state = eval(input.clone());
    let part1 = match state[Node::from(&ROOT).0] {
        Const(n) => n,
        _ => unreachable!(),
    };

    let mut state = input;
    state[Node::from(&HUMN).0] = Var;
    state[Node::from(&ROOT).0].make_op(Eql);
    state = simplify(state);

    let part2 = match state[Node::from(&ROOT).0] {
        Op(l, Eql, r) => match (state[l.0], state[r.0]) {
            (Const(n), _) => n,
            (_, Const(n)) => n,
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
