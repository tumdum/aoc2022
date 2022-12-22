use anyhow::Result;
use itertools::Itertools;
use maplit::hashmap;
use std::collections::{HashSet,BTreeSet, BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::fmt::{Error,Formatter,Debug};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Move(i32),
    Left,
    Right,
}
use Step::*;

fn parse_path(s: &str) -> Vec<Step> {
    let s = s.replace('R', " R ");
    let s = s.replace('L', " L ");
    // dbg!(&s);
    s.split(' ')
        .map(|s| match s.parse::<i32>() {
            Ok(n) => Move(n),
            _ => {
                if s == "L" {
                    Left
                } else {
                    Right
                }
            }
        })
        .collect()
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct P {
    row: isize,
    col: isize,
}

impl Debug for P {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "P({},{})", self.row, self.col)
    }
}

type Dir = P;

const RIGHT: P = P { row: 0, col: 1 };
const LEFT: P = P { row: 0, col: -1 };
const UP: P = P { row: -1, col: 0 };
const DOWN: P = P { row: 1, col: 0 };

const DIRS: [P; 4] = [UP, DOWN, LEFT, RIGHT];

impl P {
    fn get(self, m: &[Vec<char>]) -> Option<char> {
        if self.row < 0 || self.col < 0 {
            None
        } else {
            m.get(self.row as usize)
                .and_then(|row| row.get(self.col as usize))
                .copied()
        }
    }
    fn add(self, other: Self) -> Self {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
    fn sub(self, other: Self) -> Self {
        Self {
            row: self.row - other.row,
            col: self.col - other.col,
        }
    }
    fn left(self) -> Self {
        if self == RIGHT {
            return UP;
        }
        if self == UP {
            return LEFT;
        }
        if self == LEFT {
            return DOWN;
        }
        if self == DOWN {
            return RIGHT;
        }
        unreachable!()
    }
    fn right(self) -> Self {
        if self == RIGHT {
            return DOWN;
        }
        if self == DOWN {
            return LEFT;
        }
        if self == LEFT {
            return UP;
        }
        if self == UP {
            return RIGHT;
        }
        unreachable!()
    }

    fn next2(self, dir: Self, m: &[Vec<char>], cube: &HashMap<(P3,P3), (char,P)>, p_to_p3: &HashMap<P,P3>, orient_per_p: &HashMap<P, Orient>) -> Option<(P,P)> {
        let next = self.add(dir);
        match next.get(m) {
            Some('.') => return Some((next,dir)),
            Some('#') => return None,
            v => {
                let p3 = p_to_p3.get(&self).unwrap();
                // dbg!(v, self, p3);
                let orient = orient_per_p.get(&self).unwrap();
                let next_normal = if dir == UP {
                    orient.down.inv()
                } else if dir == DOWN {
                    orient.down
                } else if dir == LEFT {
                    orient.right.inv()
                } else if dir == RIGHT {
                    orient.right
                } else {
                    todo!()
                };
                let tmp_p3 = p3.add(orient.normal.inv());
                println!("{:?} + {:?} -> {:?} {}", p3, orient.normal.inv(), tmp_p3, next_normal.name());
                let tmp_p : P = cube.get(&(tmp_p3, next_normal)).unwrap().1;
                let (value, next_p2) = cube.get(&(*p3, next_normal)).unwrap();
                let new_dir = tmp_p.sub(*next_p2);
                match value {
                    '.' => return Some((*next_p2, new_dir)),
                    '#' => return None,
                    _ => todo!(),
                }
            }
            // v => todo!("{:?}: {:?}", next, v),
        }
    }
    fn next(self, dir: Self, m: &[Vec<char>]) -> Option<P> {
        let next = self.add(dir);
        match next.get(m) {
            Some('.') => Some(next),
            Some('#') => None,
            v => {
                let mut cur = if dir == RIGHT {
                    P {
                        row: self.row,
                        col: 0,
                    }
                } else if dir == DOWN {
                    P {
                        row: 0,
                        col: self.col,
                    }
                } else if dir == LEFT {
                    P {
                        row: self.row,
                        col: (m[self.row as usize].len() - 1) as isize,
                    }
                } else if dir == UP {
                    P {
                        row: (m.len() - 1) as isize,
                        col: self.col,
                    }
                } else {
                    todo!();
                };

                // dbg!(next, v, cur, dir, m.len());
                loop {
                    let cand = m[cur.row as usize][cur.col as usize];
                    if cand == '.' {
                        return Some(cur);
                    } else if cand == '#' {
                        return None;
                    } else {
                        // todo!()
                    }
                    cur = cur.add(dir);
                }
                unreachable!()
            }
            // v => todo!("{:?}: {:?}", next, v),
        }
    }
    fn name(self) -> String {
        if self == LEFT {
            return "LEFT".to_owned();
        };
        if self == RIGHT {
            return "RIGHT".to_owned();
        };
        if self == UP {
            return "UP".to_owned();
        };
        if self == DOWN {
            return "DOWN".to_owned();
        };
        format!("{self:?}")
    }
}

fn parse_map(m: &[Vec<char>]) {
    for row in 0..m.len() {
        for col in 0..m[row].len() {
            let p = P {
                row: row as isize,
                col: col as isize,
            };
            let c: Option<char> = p.get(m);
        }
    }
}

fn find_start(m: &[Vec<char>]) -> P {
    for i in 0..m[0].len() {
        if m[0][i] == '.' {
            return P {
                row: 0,
                col: i as isize,
            };
        }
    }
    unreachable!();
}

fn run_path(m: &[Vec<char>], path: &[Step]) {
    let mut pos: P = dbg!(find_start(m));
    let mut dir: P = RIGHT;
    for step in path {
        let old_pos = pos;
        let old_dir = dir;
        match step {
            Left => dir = dir.left(),
            Right => dir = dir.right(),
            Move(n) => {
                for _ in 0..*n {
                    if let Some(next) = pos.next(dir, m) {
                        pos = next;
                    } else {
                        break;
                    }
                }
            }
        }
        // println!("{old_pos:?} {old_dir:?} -> {step:?} -> {pos:?} {dir:?}");
    }
    // dbg!(pos, dir);
    let mut facing = hashmap! {
        RIGHT => 0,
        DOWN => 1,
        LEFT => 2,
        UP => 3,
    };
    dbg!((pos.row + 1) * 1000 + (pos.col + 1) * 4 + *facing.get(&dir).unwrap());
}

fn run_path2(m: &[Vec<char>], path: &[Step], cube: &HashMap<(P3,P3), (char,P)>, p_to_p3: &HashMap<P, P3>, orient_per_p: &HashMap<P, Orient>) {
    let mut pos: P = find_start(m);
    let mut dir: P = RIGHT;
    for step in path {
        let old_pos = pos;
        let old_dir = dir;
        match step {
            Left => dir = dir.left(),
            Right => dir = dir.right(),
            Move(n) => {
                for _ in 0..*n {
                    if let Some((next, next_dir)) = pos.next2(dir, m, cube, p_to_p3, orient_per_p) {
                        pos = next;
                        dir = next_dir;
                    } else {
                        break;
                    }
                }
            }
        }
        println!("{old_pos:?} {old_dir:?} -> {step:?} -> {pos:?} {dir:?}");
    }
    let mut facing = hashmap! {
        RIGHT => 0,
        DOWN => 1,
        LEFT => 2,
        UP => 3,
    };
    dbg!((pos.row + 1) * 1000 + (pos.col + 1) * 4 + *facing.get(&dir).unwrap());
}

// Side pos -> Map pos
type Side = HashMap<P, P>;

fn get_side_from(start: P, size: usize, m: &[Vec<char>]) -> Option<Side> {
    let mut ret: HashMap<P, P> = Default::default();
    for row in (start.row as usize)..(start.row as usize + size) {
        for col in (start.col as usize)..(start.col as usize + size) {
            let map_pos = P {
                row: row as isize,
                col: col as isize,
            };
            if let Some(c) = map_pos.get(m) {
                if c == ' ' {
                    return None;
                }
                let side_pos = P {
                    row: map_pos.row - start.row,
                    col: map_pos.col - start.col,
                };
                ret.insert(side_pos, map_pos);
            }
        }
    }
    if ret.len() == 0 {
        None
    } else {
        Some(ret)
    }
}

fn split_into_sides(m: &[Vec<char>], size: usize) -> BTreeMap<P, Side> {
    let mut sides: BTreeMap<P, Side> = Default::default();
    for row in 0..4 {
        for col in 0..4 {
            let start = P {
                row: (row * size) as isize,
                col: (col * size) as isize,
            };
            if let Some(side) = get_side_from(start, size, m) {
                sides.insert(
                    P {
                        row: row as isize,
                        col: col as isize,
                    },
                    side,
                );
            }
        }
    }
    sides
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct P3 {
    x: isize,
    y: isize,
    z: isize,
}

impl P3 {
    fn name(self) -> String {
        if self == UP3 { return "UP".to_owned(); }
        if self == DOWN3 { return "DOWN".to_owned(); }
        if self == LEFT3 { return "LEFT".to_owned(); }
        if self == RIGHT3 { return "RIGHT".to_owned(); }
        if self == IN3 { return "IN".to_owned(); }
        if self == OUT3 { return "OUT".to_owned(); }
        format!("{self:?}")
    }
    fn add(self, o: Self) -> P3 {
        P3 {
            x: self.x+o.x,
            y: self.y+o.y,
            z: self.z+o.z,
        }
    }
    fn inv(self) -> P3 {
        P3 {
            x: self.x * -1,
            y: self.y * -1,
            z: self.z * -1,
        }
    }
    fn down(self) -> P3 {
        if self == UP3 { return IN3}
        if self == IN3 { return DOWN3}
        if self == LEFT3 { return OUT3 }
        if self == RIGHT3 { return RIGHT3 }
        if self == DOWN3 { return OUT3 }
        unreachable!("{}", self.name())
    }
    fn ddown(self) -> P3 {
        if self == IN3 { return DOWN3 }
        if self == DOWN3 { return OUT3 }
        if self == OUT3 { return RIGHT3 }
        todo!("{}", self.name())
    }
    fn rdown(self) -> P3 {
        if self == RIGHT3 { return RIGHT3 }
        if self == DOWN3 { return DOWN3 }
        todo!("{}", self.name())
    }
    fn left(self) -> P3 {
        if self == IN3 { return LEFT3 }
        if self == LEFT3 { return OUT3 }
        if self == DOWN3 { return LEFT3 }
        unreachable!("{}", self.name())
    }
    fn right(self) -> P3 {
        if self == DOWN3 { return RIGHT3 }
        if self == UP3 { return RIGHT3 }
        unreachable!("{}", self.name())
    }
    fn dleft(self) -> P3 {
        if self == DOWN3 { return DOWN3 }
        if self == OUT3 { return OUT3 } // new
        unreachable!("{}", self.name())
    }
    fn rleft(self) -> P3 {
        // if self == RIGHT3 { return IN3 } // example
        if self == RIGHT3 { return DOWN3 } // new
        if self == IN3 { return LEFT3 }
        unreachable!("{}", self.name())
    }
    fn dright(self) -> P3 {
        if self == OUT3 { return OUT3 }
        if self == IN3 { return IN3 } // new
        unreachable!("{}", self.name())
    }
    fn rright(self) -> P3 {
        // if self == RIGHT3 { return UP3 } // example
        if self == RIGHT3 { return DOWN3 } // new
        unreachable!("{}", self.name())
    }

    fn cross(self, o: P3) -> P3 {
        let mut ret : P3 = P3{ x: 0, y: 0, z: 0 };
        ret.x = self.y * o.z - self.z * o.y;
        ret.y = -(self.x * o.z - self.z * o.x);
        ret.z = self.x * o.y - self.y * o.x;
        ret
    }
}

const UP3: P3 = P3 { x: 0, y: -1, z: 0 };
const DOWN3: P3 = P3 { x: 0, y: 1, z: 0 };
const LEFT3: P3 = P3 { x: -1, y: 0, z: 0 };
const RIGHT3: P3 = P3 { x: 1, y: 0, z: 0 };
const IN3: P3 = P3 { x: 0, y: 0, z: -1 };
const OUT3: P3 = P3 { x: 0, y: 0, z: 1 };

#[derive(Debug, Clone, Copy)]
struct Orient {
    down: P3,
    right: P3,
    normal: P3,
}

impl Orient {
    fn print(&self) -> String {
        // assert_eq!(self.normal, self.down.cross(self.right), "{:?}", self);
        // assert_eq!(self.right, self.normal.cross(self.down));
        format!("Orient{{down: {}, right: {}, normal: {}}}", self.down.name(), self.right.name(), self.normal.name())
    }
    fn check(&self, name: &P) {
        assert_eq!(self.normal, self.down.cross(self.right), "{name:?}: {:?}", self);
        assert_eq!(self.right, self.normal.cross(self.down));
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();
    let path = input.last().unwrap();
    let mut map: Vec<Vec<char>> = input[..input.len() - 2]
        .iter()
        .map(|l| l.chars().collect())
        .collect();
    let max_w = map.iter().map(|row| row.len()).max().unwrap();
    let min_w = map.iter().map(|row| row.len()).min().unwrap();
    for i in 0..map.len() {
        while map[i].len() != max_w {
            map[i].push(' ');
        }
    }
    let max_w = map.iter().map(|row| row.len()).max().unwrap();
    let min_w = map.iter().map(|row| row.len()).min().unwrap();
    let path = parse_path(path);

    parse_map(&map);

    let s = Instant::now();

    run_path(&map, &path);

    let size = if map.len() > 50 { 50 } else { 4 };

    let sides: BTreeMap<P, Side> = split_into_sides(&map, size);
    for p in sides.keys() {
        println!("{p:?}");
    }
    // Side -> Normal
    let mut normals: BTreeMap<P, Orient> = Default::default();
    normals.insert(*sides.keys().find(|p| p.row == 0).unwrap(), Orient{ down: IN3, right: RIGHT3, normal: UP3 });
    // println!("normals: {:?}", normals.iter().map(|(p,d)| (p,d.name())).collect_vec());
    let mut todo : BTreeSet<P> = sides.keys().copied().collect();
    todo.remove(sides.keys().find(|p| p.row == 0).unwrap());
    while todo.len() > 0 {
        println!("todo: {}, done: {}", todo.len(), normals.len());
        println!("normals: {:?}", normals.iter().map(|(p,o)| (p,o.print())).collect_vec());
        normals.iter().for_each(|(p,o)| o.check(p));
        let mut to_add = vec![];
        let mut to_rem = vec![];
        'out: for to_check in &todo {
            for (done, orient) in &normals {
                if done.add(DOWN) == *to_check {
                    println!("{done:?} -> {to_check:?}");
                    let next_normal = orient.normal.down();
                    let next_down = orient.down.ddown();
                    println!("{done:?}->{to_check:?}: down rotated by down: {} -> {}", orient.down.name(), next_down.name());
                    let next_right = orient.right.rdown();
                    println!("{done:?}->{to_check:?}: right rotated by down: {} -> {}", orient.right.name(), next_right.name());
                    // println!("\tdown: {to_check:?} -> {}", next_normal.name());
                    to_add.push((*to_check, Orient{ normal: next_normal, down: next_down, right: next_right}));
                    to_rem.push(*to_check);
                    break 'out;
                }
                if done.add(LEFT) == *to_check {
                    println!("{done:?} -> {to_check:?}");
                    let next_normal = orient.normal.left();
                    let next_down = orient.down.dleft();
                    println!("{done:?}->{to_check:?}: down rotated by left: {} -> {}", orient.down.name(), next_down.name());
                    let next_right = orient.right.rleft();
                    println!("{done:?}->{to_check:?}: right rotated by left: {} -> {}", orient.right.name(), next_right.name());
                    to_add.push((*to_check, Orient{ normal: next_normal, down: next_down, right: next_right}));
                    to_rem.push(*to_check);
                    break 'out;
                }
                if done.add(RIGHT) == *to_check {
                    println!("{done:?} -> {to_check:?}");
                    let next_normal = orient.normal.right();
                    let next_down = orient.down.dright();
                    println!("{done:?}->{to_check:?}: down rotated by right: {} -> {}", orient.down.name(), next_down.name());
                    let next_right = orient.right.rright();
                    println!("{done:?}->{to_check:?}: right rotated by right: {} -> {}", orient.right.name(), next_right.name());
                    to_add.push((*to_check, Orient{ normal: next_normal, down: next_down, right: next_right} ));
                    to_rem.push(*to_check);
                    break 'out;
                }
            }
        }
        for (p, n) in to_add {
            normals.insert(p, n);
        }
        for p in to_rem {
            todo.remove(&p);
        }
    }
    normals.iter().for_each(|(p,o)| o.check(p));
    println!();
    for (p, orient) in &normals {
        println!("{p:?}: {}", orient.print());
    }

    // (Pos, Normal) -> content
    let max = size as isize - 1;
    let mut cube : HashMap<(P3, P3), (char,P)> = Default::default();
    let mut p_to_p3 : HashMap<P, P3> = Default::default();
    for (p, points) in &sides {
        let orient = normals.get(&p).unwrap();
        // println!("{:?}: {}", p, orient.print());
        let mut row_start : P3 = if orient.normal == UP3 {
            P3{x: 0, y: 0, z: 0} // new
            // P3{x: 0, y: 0, z: 0} // example
        } else if orient.normal == OUT3 {
            P3{x: 0, y: 0, z: 0} // new
            // P3{x: max, y: 0, z: 0} // example
        } else if orient.normal == LEFT3 {
            P3 {x: 0, y: 0, z: -max} // new
            // P3{x: 0, y: 0, z: 0} // example
        } else if orient.normal == IN3 {
            P3 {x: 0, y: 0, z: -max} // new
            // P3{x: 0, y: 0, z: -max} // example
        } else if orient.normal == DOWN3 {
            P3 {x: 0, y: max, z: -max} // new
            // P3{x: 0, y: max, z: -max} // example
        } else if orient.normal == RIGHT3 {
            P3{x: max, y: 0, z: 0} // new
            // P3{x: max, y: max, z: -max} // example
        } else {
            todo!("{}", orient.print())
        };
        println!("{:?}: start: {:?}", p, row_start);
        let (min_x, max_x) = points.keys().map(|p| p.col).minmax().into_option().unwrap();
        let (min_y, max_y) = points.keys().map(|p| p.row).minmax().into_option().unwrap();
        assert_eq!(0, min_x);
        assert_eq!(size as isize -1, max_x);
        assert_eq!(0, min_y);
        assert_eq!(size as isize -1, max_y);
        for row in 0..size {
            let mut cube_pos = row_start;
            for col in 0..size {
                let tmp_pos = P{row: row as isize, col: col as isize};
                let tmp_pos2 = points.get(&tmp_pos).unwrap();
                let value = tmp_pos2.get(&map).unwrap();
                // println!("{:?} -> {:?} -> {} at {:?}", tmp_pos, tmp_pos2, value, cube_pos);
                cube.insert((cube_pos, orient.normal), (value,*tmp_pos2));
                p_to_p3.insert(*tmp_pos2, cube_pos);
                cube_pos = cube_pos.add(orient.right);
            }
            row_start = row_start.add(orient.down);
            // println!("row start {:?}", row_start);
        }
    }

    let mut orient_per_p : HashMap<P, Orient> = Default::default();
    for (p, orient) in normals {
        let points_with_same_normal = sides.get(&p).unwrap();
        for (_side_pos, map_pos) in points_with_same_normal {
            orient_per_p.insert(*map_pos, orient);
        }
    }
    // dbg!(orient_per_p.len());

    run_path2(&map, &path, &cube, &p_to_p3, &orient_per_p);

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
