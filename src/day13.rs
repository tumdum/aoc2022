use anyhow::Result;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone)]
enum Packet {
    Num(u8),
    List(Vec<Packet>),
}

fn in_right_order(l: &Packet, r: &Packet) -> Option<bool> {
    use Packet::*;
    match (l, r) {
        (Num(l), Num(r)) => match l.cmp(r) {
            Ordering::Less => Some(true),
            Ordering::Equal => None,
            Ordering::Greater => Some(false),
        },
        (List(l), List(r)) => in_right_order_list(l, r),
        (Num(_), List(r)) => in_right_order_list(&[l.clone()], r),
        (List(l), Num(_)) => in_right_order_list(l, &[r.clone()]),
    }
}

fn in_right_order_list(l: &[Packet], r: &[Packet]) -> Option<bool> {
    let mut i = 0;
    loop {
        if i >= l.len() && i >= r.len() {
            return None;
        }
        if i >= l.len() {
            return Some(true);
        }
        if i >= r.len() {
            return Some(false);
        }
        match in_right_order(&l[i], &r[i]) {
            None => {}
            Some(b) => return Some(b),
        }
        i += 1
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
    (Packet::Num(ret), &l[i..])
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
    (Packet::List(v), &l[1..])
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
        .filter(|(_, (l, r))| in_right_order(l, r).unwrap())
        .map(|(i, _)| i + 1)
        .sum();

    let mut input: Vec<Packet> = input.into_iter().flat_map(|(l, r)| [l, r]).collect();

    let d1 = parse("[[2]]");
    let d2 = parse("[[6]]");
    input.push(d1.clone());
    input.push(d2.clone());
    input.sort_by(|l, r| {
        if let Some(true) = in_right_order(l, r) {
            return Ordering::Less;
        }
        if let Some(true) = in_right_order(r, l) {
            return Ordering::Greater;
        }
        unreachable!()
    });

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
        assert_eq!(Packet::Num(42), n);
        assert_eq!(",test".to_owned(), rest.iter().collect::<String>());

        let input = "42".chars().collect::<Vec<_>>();
        let (n, rest) = parse_num(&input);
        assert_eq!(Packet::Num(42), n);
        assert_eq!("".to_owned(), rest.iter().collect::<String>());
    }

    #[test]
    fn parse_list_test() {
        let input = "[42,13]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(Packet::List(vec![Packet::Num(42), Packet::Num(13),]), l);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_list_nested() {
        let input = "[42,[13,3]]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(
            Packet::List(vec![
                Packet::Num(42),
                Packet::List(vec![Packet::Num(13), Packet::Num(3),]),
            ]),
            l
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_list_nested2() {
        let input = "[[42],[13,3]]".chars().collect::<Vec<_>>();
        let (l, rest) = parse_list(&input);
        assert_eq!(
            Packet::List(vec![
                Packet::List(vec![Packet::Num(42)]),
                Packet::List(vec![Packet::Num(13), Packet::Num(3),]),
            ]),
            l
        );
        assert!(rest.is_empty());
    }
}
