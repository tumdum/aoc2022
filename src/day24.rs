use anyhow::Result;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct P {
    row: i16,
    col: i16,
}

impl P {
    fn get<'a>(&self, m: &'a Map) -> Option<&'a [Item]> {
        m.get(self.row as usize)
            .and_then(|row| row.get(self.col as usize))
            .map(|v| v.as_slice())
    }
    fn set(&self, m: &mut Map, val: Item) {
        debug_assert!(self.row >= 0 && self.col >= 0);
        debug_assert!(self.row < m.len() as i16 && self.col < m[0].len() as i16);
        m[self.row as usize][self.col as usize].push(val);
    }
    fn add(self, o: Self) -> P {
        P {
            row: self.row + o.row,
            col: self.col + o.col,
        }
    }
    fn is_not_wind(&self, m: &Map) -> bool {
        self.get(m)
            .map(|contents| contents.iter().all(|item| item != &Wall))
            .unwrap_or(false)
    }
    fn is_free(&self, m: &Map) -> bool {
        self.get(m)
            .map(|contents| contents.is_empty())
            .unwrap_or(false)
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
                    Wall => print!("#"),
                    _ => todo!(),
                }
            }
        }
        println!();
    }
}

fn clean_map(m: &Map) -> Map {
    let rows = m.len();
    let cols = m[0].len();
    let mut base = vec![vec![vec![]; cols]; rows];
    base[0] = m[0].clone();
    base[rows - 1] = m[rows - 1].clone();
    for row in 0..rows {
        base[row][0] = vec![Wall];
        base[row][cols - 1] = vec![Wall];
    }
    base
}

fn next_wind(m: &Map) -> Map {
    let mut next = clean_map(m);
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
                    if next_wind_pos.is_not_wind(&next) {
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

fn find_path(
    start: P,
    end: P,
    start_time: usize,
    seen: &mut Vec<Vec<Vec<bool>>>,
    states: &[Map],
) -> usize {
    let mut todo: VecDeque<(P, usize)> = Default::default();
    todo.push_back((start, start_time));

    while let Some((p, t)) = todo.pop_front() {
        if p == end {
            return t;
        }
        for next_pos in MOVES.into_iter().map(|dir| p.add(dir)) {
            let next_time = t + 1;
            if next_pos.is_free(&states[next_time % states.len()]) {
                if seen[next_pos.row as usize][next_pos.col as usize][next_time] == true {
                    continue;
                }
                seen[next_pos.row as usize][next_pos.col as usize][next_time] = true;
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
    let start_map = map;

    let start = P { row: 0, col: 1 };
    let end = P {
        row: start_map.len() as i16 - 1,
        col: start_map
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
    states.push(start_map.clone());
    loop {
        let next = next_wind(states.last().unwrap());
        if next == start_map {
            break;
        }
        states.push(next);
    }

    // row / col / time
    let mut seen: Vec<Vec<Vec<bool>>> =
        vec![vec![vec![false; 1000]; states[0][0].len()]; states[0].len()];

    let part1 = find_path(start, end, 0, &mut seen, &states);
    let back = find_path(end, start, part1, &mut seen, &states);
    let part2 = find_path(start, end, back, &mut seen, &states);

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
