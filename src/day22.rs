use anyhow::Result;
use itertools::{iproduct, Itertools};
use maplit::hashmap;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::collections::BTreeMap;
use std::fmt::{Debug, Error, Formatter};
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Move(i32),
    Left,
    Right,
}
use Step::*;

fn parse_path(s: &str) -> Vec<Step> {
    let s = s.replace('R', " R ").replace('L', " L ");
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

const RIGHT: P = P { row: 0, col: 1 };
const LEFT: P = P { row: 0, col: -1 };
const UP: P = P { row: -1, col: 0 };
const DOWN: P = P { row: 1, col: 0 };

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

    fn next2(
        self,
        dir: Self,
        m: &[Vec<char>],
        cube: &HashMap<(P3, P3), (char, P)>,
        p_to_p3: &HashMap<P, P3>,
        orient_per_p: &HashMap<P, Orient>,
    ) -> Option<(P, P)> {
        let next = self.add(dir);
        match next.get(m) {
            Some('.') => Some((next, dir)),
            Some('#') => None,
            _ => {
                let p3 = p_to_p3.get(&self).unwrap();
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
                let next_from_p3 = p3.add(orient.normal.inv());
                let next_from_p3_as_p: P = cube.get(&(next_from_p3, next_normal)).unwrap().1;
                let (value, next_p2) = cube.get(&(*p3, next_normal)).unwrap();
                let new_dir = next_from_p3_as_p.sub(*next_p2);
                match value {
                    '.' => Some((*next_p2, new_dir)),
                    '#' => None,
                    _ => todo!(),
                }
            }
        }
    }
    fn next(self, dir: Self, m: &[Vec<char>]) -> Option<P> {
        let next = self.add(dir);
        match next.get(m) {
            Some('.') => Some(next),
            Some('#') => None,
            _ => {
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

                loop {
                    let cand = m[cur.row as usize][cur.col as usize];
                    if cand == '.' {
                        return Some(cur);
                    } else if cand == '#' {
                        return None;
                    }
                    cur = cur.add(dir);
                }
            }
        }
    }

    #[allow(unused)]
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

fn run_path(m: &[Vec<char>], path: &[Step]) -> usize {
    let mut pos: P = find_start(m);
    let mut dir: P = RIGHT;
    for step in path {
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
    }
    let facing = hashmap! {
        RIGHT => 0,
        DOWN => 1,
        LEFT => 2,
        UP => 3,
    };
    ((pos.row + 1) * 1000 + (pos.col + 1) * 4 + *facing.get(&dir).unwrap()) as usize
}

fn run_path2(
    m: &[Vec<char>],
    path: &[Step],
    cube: &HashMap<(P3, P3), (char, P)>,
    p_to_p3: &HashMap<P, P3>,
    orient_per_p: &HashMap<P, Orient>,
) -> usize {
    let mut pos: P = find_start(m);
    let mut dir: P = RIGHT;
    for step in path {
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
    }
    let facing = hashmap! {
        RIGHT => 0,
        DOWN => 1,
        LEFT => 2,
        UP => 3,
    };
    ((pos.row + 1) * 1000 + (pos.col + 1) * 4 + *facing.get(&dir).unwrap()) as usize
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
    if ret.is_empty() {
        None
    } else {
        Some(ret)
    }
}

fn split_into_sides(m: &[Vec<char>], size: usize) -> BTreeMap<P, Side> {
    iproduct!(0..4, 0..4)
        .map(|(row, col)| P {
            row: (row * size) as isize,
            col: (col * size) as isize,
        })
        .flat_map(|start| {
            get_side_from(start, size, m).map(|side| {
                (
                    P {
                        row: start.row / size as isize,
                        col: start.col / size as isize,
                    },
                    side,
                )
            })
        })
        .collect()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct P3 {
    x: isize,
    y: isize,
    z: isize,
}

impl P3 {
    #[allow(unused)]
    fn name(self) -> String {
        if self == UP3 {
            return "UP".to_owned();
        }
        if self == DOWN3 {
            return "DOWN".to_owned();
        }
        if self == LEFT3 {
            return "LEFT".to_owned();
        }
        if self == RIGHT3 {
            return "RIGHT".to_owned();
        }
        if self == IN3 {
            return "IN".to_owned();
        }
        if self == OUT3 {
            return "OUT".to_owned();
        }
        format!("{self:?}")
    }
    fn add(self, o: Self) -> P3 {
        P3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
    fn inv(self) -> P3 {
        P3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
    fn down(self) -> P3 {
        if self == UP3 {
            return IN3;
        }
        if self == IN3 {
            return DOWN3;
        }
        if self == LEFT3 {
            return OUT3;
        }
        if self == RIGHT3 {
            return RIGHT3;
        }
        if self == DOWN3 {
            return OUT3;
        }
        unreachable!("{}", self.name())
    }
    fn ddown(self) -> P3 {
        if self == IN3 {
            return DOWN3;
        }
        if self == DOWN3 {
            return OUT3;
        }
        if self == OUT3 {
            return RIGHT3;
        }
        todo!("{}", self.name())
    }
    fn rdown(self) -> P3 {
        if self == RIGHT3 {
            return RIGHT3;
        }
        if self == DOWN3 {
            return DOWN3;
        }
        todo!("{}", self.name())
    }
    fn left(self) -> P3 {
        if self == IN3 {
            return LEFT3;
        }
        if self == LEFT3 {
            return OUT3;
        }
        if self == DOWN3 {
            return LEFT3;
        }
        unreachable!("{}", self.name())
    }
    fn right(self) -> P3 {
        if self == DOWN3 {
            return RIGHT3;
        }
        if self == UP3 {
            return RIGHT3;
        }
        unreachable!("{}", self.name())
    }
    fn down_by_left(self) -> P3 {
        if self == DOWN3 {
            return DOWN3;
        }
        if self == OUT3 {
            return OUT3;
        } // new
        unreachable!("{}", self.name())
    }
    fn right_by_left(self) -> P3 {
        // if self == RIGHT3 { return IN3 } // example
        if self == RIGHT3 {
            return DOWN3;
        } // new
        if self == IN3 {
            return LEFT3;
        }
        unreachable!("{}", self.name())
    }
    fn down_by_right(self) -> P3 {
        if self == OUT3 {
            return OUT3;
        }
        if self == IN3 {
            return IN3;
        } // new
        unreachable!("{}", self.name())
    }
    fn right_by_right(self) -> P3 {
        // if self == RIGHT3 { return UP3 } // example
        if self == RIGHT3 {
            return DOWN3;
        } // new
        unreachable!("{}", self.name())
    }

    fn cross(self, o: P3) -> P3 {
        let mut ret: P3 = P3 { x: 0, y: 0, z: 0 };
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
        format!(
            "Orient{{down: {}, right: {}, normal: {}}}",
            self.down.name(),
            self.right.name(),
            self.normal.name()
        )
    }
    fn check(&self, name: &P) {
        debug_assert_eq!(
            self.normal,
            self.down.cross(self.right),
            "{name:?}: {:?}",
            self
        );
        debug_assert_eq!(self.right, self.normal.cross(self.down));
    }
}

fn find_normals_for_sides(sides: &BTreeMap<P, Side>) -> HashMap<P, Orient> {
    let mut normals: HashMap<P, Orient> = Default::default();
    normals.insert(
        *sides.keys().find(|p| p.row == 0).unwrap(),
        Orient {
            down: IN3,
            right: RIGHT3,
            normal: UP3,
        },
    );
    let mut todo: HashSet<P> = sides.keys().copied().collect();
    todo.remove(sides.keys().find(|p| p.row == 0).unwrap());
    while !todo.is_empty() {
        normals.iter().for_each(|(p, o)| o.check(p));
        let mut to_add = vec![];
        let mut to_rem = vec![];
        'out: for to_check in &todo {
            for (done, orient) in &normals {
                if done.add(DOWN) == *to_check {
                    let next_normal = orient.normal.down();
                    let next_down = orient.down.ddown();
                    let next_right = orient.right.rdown();
                    to_add.push((
                        *to_check,
                        Orient {
                            normal: next_normal,
                            down: next_down,
                            right: next_right,
                        },
                    ));
                    to_rem.push(*to_check);
                    break 'out;
                }
                if done.add(LEFT) == *to_check {
                    let next_normal = orient.normal.left();
                    let next_down = orient.down.down_by_left();
                    let next_right = orient.right.right_by_left();
                    to_add.push((
                        *to_check,
                        Orient {
                            normal: next_normal,
                            down: next_down,
                            right: next_right,
                        },
                    ));
                    to_rem.push(*to_check);
                    break 'out;
                }
                if done.add(RIGHT) == *to_check {
                    let next_normal = orient.normal.right();
                    let next_down = orient.down.down_by_right();
                    let next_right = orient.right.right_by_right();
                    to_add.push((
                        *to_check,
                        Orient {
                            normal: next_normal,
                            down: next_down,
                            right: next_right,
                        },
                    ));
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
    normals.iter().for_each(|(p, o)| o.check(p));
    normals
}

fn create_cube(
    map: &[Vec<char>],
    sides: &BTreeMap<P, Side>,
    normals: &HashMap<P, Orient>,
    size: usize,
) -> (HashMap<(P3, P3), (char, P)>, HashMap<P, P3>) {
    let max = size as isize - 1;
    let mut cube: HashMap<(P3, P3), (char, P)> = Default::default();
    let mut p_to_p3: HashMap<P, P3> = Default::default();
    for (p, points) in sides {
        let orient = normals.get(p).unwrap();
        let mut row_start: P3 = if orient.normal == UP3 {
            P3 { x: 0, y: 0, z: 0 } // new
                                    // P3{x: 0, y: 0, z: 0} // example
        } else if orient.normal == OUT3 {
            P3 { x: 0, y: 0, z: 0 } // new
                                    // P3{x: max, y: 0, z: 0} // example
        } else if orient.normal == LEFT3 {
            P3 {
                x: 0,
                y: 0,
                z: -max,
            } // new
              // P3{x: 0, y: 0, z: 0} // example
        } else if orient.normal == IN3 {
            P3 {
                x: 0,
                y: 0,
                z: -max,
            } // new
              // P3{x: 0, y: 0, z: -max} // example
        } else if orient.normal == DOWN3 {
            P3 {
                x: 0,
                y: max,
                z: -max,
            } // new
              // P3{x: 0, y: max, z: -max} // example
        } else if orient.normal == RIGHT3 {
            P3 { x: max, y: 0, z: 0 } // new
                                      // P3{x: max, y: max, z: -max} // example
        } else {
            todo!("{}", orient.print())
        };
        let (min_x, max_x) = points.keys().map(|p| p.col).minmax().into_option().unwrap();
        let (min_y, max_y) = points.keys().map(|p| p.row).minmax().into_option().unwrap();
        assert_eq!(0, min_x);
        assert_eq!(size as isize - 1, max_x);
        assert_eq!(0, min_y);
        assert_eq!(size as isize - 1, max_y);
        for row in 0..size {
            let mut cube_pos = row_start;
            for col in 0..size {
                let tmp_pos = P {
                    row: row as isize,
                    col: col as isize,
                };
                let tmp_pos2 = points.get(&tmp_pos).unwrap();
                let value = tmp_pos2.get(map).unwrap();
                cube.insert((cube_pos, orient.normal), (value, *tmp_pos2));
                p_to_p3.insert(*tmp_pos2, cube_pos);
                cube_pos = cube_pos.add(orient.right);
            }
            row_start = row_start.add(orient.down);
        }
    }
    (cube, p_to_p3)
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();
    let path = input.last().unwrap();
    let mut map: Vec<Vec<char>> = input[..input.len() - 2]
        .iter()
        .map(|l| l.chars().collect())
        .collect();
    let max_w = map.iter().map(|row| row.len()).max().unwrap();
    for row in &mut map {
        while row.len() != max_w {
            row.push(' ');
        }
    }
    let path = parse_path(path);

    let s = Instant::now();

    let part1 = run_path(&map, &path);

    let size = if map.len() > 50 { 50 } else { 4 };

    let sides: BTreeMap<P, Side> = split_into_sides(&map, size);

    // Side -> Normal
    let normals: HashMap<P, Orient> = find_normals_for_sides(&sides);

    let (cube, p_to_p3) = create_cube(&map, &sides, &normals, size);

    let orient_per_p: HashMap<P, Orient> = normals
        .iter()
        .flat_map(|(p, orient)| sides.get(p).map(|side| (side, orient)))
        .flat_map(|(side, orient)| side.iter().map(|(_, map_pos)| (*map_pos, *orient)))
        .collect();

    let part2 = run_path2(&map, &path, &cube, &p_to_p3, &orient_per_p);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(56372, part1);
        assert_eq!(197047, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
