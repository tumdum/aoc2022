use anyhow::Result;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone, Eq)]
enum Packet {
    Num(u8),
    List(Vec<Packet>),
}

use Packet::*;

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        fn in_right_order_list(l: &[Packet], r: &[Packet]) -> Ordering {
            let mut i = 0;
            loop {
                if i >= l.len() && i >= r.len() {
                    return Ordering::Equal;
                }
                if i >= l.len() {
                    return Ordering::Less;
                }
                if i >= r.len() {
                    return Ordering::Greater;
                }
                match l[i].cmp(&r[i]) {
                    Ordering::Equal => {}
                    o => return o,
                }
                i += 1
            }
        }

        match (self, other) {
            (Num(l), Num(r)) => l.cmp(r),
            (List(l), List(r)) => in_right_order_list(l, r),
            (Num(_), List(r)) => in_right_order_list(&[self.clone()], r),
            (List(l), Num(_)) => in_right_order_list(l, &[other.clone()]),
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse(s: &str) -> Packet {
    let s = s.chars().collect::<Vec<_>>();
    let (p, rest) = parse_list(&s);
    debug_assert!(rest.is_empty());
    p
}

fn parse_num(l: &[char]) -> (Packet, &[char]) {
    let mut i = 0;
    while i < l.len() && l[i].is_ascii_digit() {
        i += 1;
    }
    assert!(i > 0);
    let mut ret = 0u8;
    let mut off = 1;
    for c in l[..i].iter().rev() {
        ret += (*c as u8 - b'0') * off;
        off *= 10;
    }
    (Num(ret), &l[i..])
}

fn parse_list(l: &[char]) -> (Packet, &[char]) {
    debug_assert_eq!('[', l[0]);

    let mut l = &l[1..];
    let mut v = Vec::with_capacity(8);

    loop {
        match l[0] {
            ']' => break,
            ',' => l = &l[1..],
            '[' => {
                let (ll, rest) = parse_list(l);
                v.push(ll);
                l = rest;
            }
            _ => {
                let (n, rest) = parse_num(l);
                v.push(n);
                l = rest;
            }
        }
    }
    debug_assert_eq!(']', l[0]);
    (List(v), &l[1..])
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();

    let input: Vec<_> = input
        .split(|l| l.is_empty())
        .map(|v| (parse(v[0]), parse(v[1])))
        .collect();

    let s = Instant::now();

    let part1: usize = input
        .iter()
        .enumerate()
        .filter(|(_, (l, r))| Ordering::Less == l.cmp(r))
        .map(|(i, _)| i + 1)
        .sum();

    let mut input: Vec<_> = input.into_iter().flat_map(|(l, r)| [l, r]).collect();

    let d1 = parse("[[2]]");
    let d2 = parse("[[6]]");
    input.push(d1.clone());
    input.push(d2.clone());
    input.sort_unstable();

    let part2: usize = input
        .into_iter()
        .enumerate()
        .filter(|(_, p)| p == &d1 || p == &d2)
        .map(|(i, _)| i + 1)
        .product();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(5720, part1);
        assert_eq!(23504, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_num_test() {
        let input = "42,test".chars().collect::<Vec<_>>();
        let (n, rest) = parse_num(&input);
        assert_eq!(Num(42), n);
        assert_eq!(",test".to_owned(), rest.iter().collect::<String>());

        let input = "42".chars().collect::<Vec<_>>();
        let (n, rest) = parse_num(&input);
        assert_eq!(Num(42), n);
        assert_eq!("".to_owned(), rest.iter().collect::<String>());
    }

    #[test]
    fn parse_list_test() {
        let input = "[42,13]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(List(vec![Num(42), Num(13),]), l);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_list_nested() {
        let input = "[42,[13,3]]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(List(vec![Num(42), List(vec![Num(13), Num(3),]),]), l);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_list_nested2() {
        let input = "[[42],[13,3]]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(
            List(vec![List(vec![Num(42)]), List(vec![Num(13), Num(3),]),]),
            l
        );
        assert!(rest.is_empty());
    }
}
