use anyhow::Result;
use rustc_hash::FxHashSet as HashSet;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct P {
    row: i16,
    col: i16,
}

impl P {
    fn get<'a>(&self, m: &'a Map) -> Option<&'a [Item]> {
        if self.row < 0 || self.col < 0 {
            return None;
        }
        m.get(self.row as usize)
            .and_then(|row| row.get(self.col as usize))
            .map(|v| v.as_slice())
    }
    fn set(&self, m: &mut Map, val: Item) {
        assert!(self.row >= 0 && self.col >= 0);
        assert!(self.row < m.len() as i16 && self.col < m[0].len() as i16);
        m[self.row as usize][self.col as usize].push(val);
    }
    fn add(self, o: Self) -> P {
        P {
            row: self.row + o.row,
            col: self.col + o.col,
        }
    }
    fn is_free_for_wind(&self, m: &Map) -> bool {
        match self.get(m) {
            None => false,
            Some(content) => {
                let ret = content.iter().all(|item| item != &Wall);
                // println!("{:?}: {:?} vs {}", self, content, ret);
                ret
            }
        }
    }
    fn is_free_for_exp(&self, m: &Map) -> bool {
        match self.get(m) {
            None => false,
            Some(content) => {
                let ret = content
                    .iter()
                    .all(|item| item != &Wall && !matches!(item, Blizzard(_)));
                ret
            }
        }
    }
}

const UP: P = P { row: -1, col: 0 };
const DOWN: P = P { row: 1, col: 0 };
const LEFT: P = P { row: 0, col: -1 };
const RIGHT: P = P { row: 0, col: 1 };
const NONE: P = P { row: 0, col: 0 };

const MOVES: [P; 5] = [UP, DOWN, LEFT, RIGHT, NONE];

impl From<u8> for P {
    fn from(b: u8) -> P {
        match b {
            b'>' => RIGHT,
            b'<' => LEFT,
            b'^' => UP,
            b'v' => DOWN,
            _ => todo!(),
        }
    }
}

type Dir = P;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Blizzard(Dir),
    Wall,
}
use Item::*;

impl Item {
    fn parse(b: u8) -> Vec<Item> {
        let mut ret = vec![];
        match b {
            b'>' | b'<' | b'^' | b'v' => ret.push(Blizzard(b.into())),
            b'#' => ret.push(Wall),
            b'.' => {}
            v => todo!("{}", v as char),
        }
        ret
    }
}

// row / col / Vec of items
type Map = Vec<Vec<Vec<Item>>>;

#[allow(unused)]
fn print(m: &Map) {
    for row in 0..m.len() {
        for col in 0..m[row].len() {
            let p = P {
                row: row as i16,
                col: col as i16,
            };
            let content = p.get(m).unwrap();
            if content.len() > 1 {
                assert!(content.iter().all(|item| matches!(item, Blizzard(_))));
                print!("{}", content.len());
            } else if content.is_empty() {
                print!(".");
            } else {
                match content[0] {
                    Blizzard(d) if d == UP => print!("^"),
                    Blizzard(d) if d == DOWN => print!("v"),
                    Blizzard(d) if d == LEFT => print!("<"),
                    Blizzard(d) if d == RIGHT => print!(">"),
                    // Expedition => print!("E"),
                    Wall => print!("#"),
                    _ => todo!(),
                }
            }
        }
        println!();
    }
}

fn copy_without_wind(m: &Map) -> Map {
    let mut ret = Vec::with_capacity(m.len());
    for row in m {
        ret.push(Vec::with_capacity(row.len()));
        for e in row {
            ret.last_mut().unwrap().push(
                e.iter()
                    .filter(|item| !matches!(item, Blizzard(_)))
                    .copied()
                    .collect(),
            );
        }
    }
    ret
}

fn next_wind(m: &Map) -> Map {
    let mut next = copy_without_wind(m);
    for row in 0..m.len() {
        for col in 0..m[row].len() {
            let p = P {
                row: row as i16,
                col: col as i16,
            };
            let content = p.get(m).unwrap();
            for wind in content.iter().filter(|item| matches!(item, Blizzard(_))) {
                if let Blizzard(dir) = wind {
                    let next_wind_pos = p.add(*dir);
                    if next_wind_pos.is_free_for_wind(&next) {
                        next_wind_pos.set(&mut next, *wind);
                    } else {
                        let next_wind_pos = if *dir == DOWN {
                            P { col: p.col, row: 1 }
                        } else if *dir == UP {
                            P {
                                col: p.col,
                                row: m.len() as i16 - 2,
                            }
                        } else if *dir == RIGHT {
                            P { col: 1, row: p.row }
                        } else if *dir == LEFT {
                            P {
                                col: m[p.row as usize].len() as i16 - 2,
                                row: p.row,
                            }
                        } else {
                            todo!("dir: {:?}", dir)
                        };
                        next_wind_pos.set(&mut next, *wind);
                    }
                }
            }
        }
    }
    next
}

fn find_path(start: P, end: P, start_time: usize, states: &[Map]) -> usize {
    let mut seen: HashSet<(P, usize)> = Default::default();

    let mut todo: VecDeque<(P, usize)> = Default::default();
    todo.push_back((start, start_time));

    while let Some((p, t)) = todo.pop_front() {
        if p == end {
            return t;
        }
        for next_pos in MOVES.into_iter().map(|dir| p.add(dir)) {
            let next_time = t + 1;
            if next_pos.is_free_for_exp(&states[next_time % states.len()]) {
                if seen.contains(&(next_pos, next_time)) {
                    continue;
                }
                seen.insert((next_pos, next_time));
                todo.push_back((next_pos, next_time));
            }
        }
    }
    unreachable!()
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();

    let mut map: Map = vec![];
    for row in input {
        map.push(vec![]);
        for element in row {
            map.last_mut().unwrap().push(Item::parse(element));
        }
    }

    let start = P { row: 0, col: 1 };
    let end = P {
        row: map.len() as i16 - 1,
        col: map
            .last()
            .unwrap()
            .iter()
            .enumerate()
            .find(|(_, v)| v.is_empty())
            .unwrap()
            .0 as i16,
    };
    let s = Instant::now();

    let mut states: Vec<Map> = vec![];
    states.push(map.clone());
    for _ in 0.. {
        let next = next_wind(states.last().unwrap());
        if next == map {
            break;
        }
        states.push(next);
    }

    let part1 = find_path(start, end, 0, &states);
    let back = find_path(end, start, part1, &states);
    let part2 = find_path(start, end, back, &states);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(332, part1);
        assert_eq!(942, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
