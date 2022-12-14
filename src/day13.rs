use anyhow::Result;
use nom::bytes::complete::tag;
use nom::character::complete::{char, u8};
use nom::{branch::alt, multi::separated_list0, sequence::delimited, IResult};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone, Eq)]
enum Packet {
    Num(u8),
    List(Vec<Packet>),
}

use Packet::*;

fn packet(i: &str) -> IResult<&str, Packet> {
    fn num(input: &str) -> IResult<&str, Packet> {
        u8(input).map(|(rest, n)| (rest, Num(n)))
    }
    fn packets(input: &str) -> IResult<&str, Vec<Packet>> {
        separated_list0(tag(","), packet)(input)
    }
    fn list(input: &str) -> IResult<&str, Packet> {
        delimited(char('['), packets, char(']'))(input).map(|(rest, l)| (rest, List(l)))
    }
    alt((list, num))(i)
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        fn in_right_order_list<T: Ord + Borrow<Packet>, U: Ord + Borrow<Packet>>(
            l: &[T],
            r: &[U],
        ) -> Ordering {
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
                match l[i].borrow().cmp(r[i].borrow()) {
                    Ordering::Equal => {}
                    o => return o,
                }
                i += 1
            }
        }

        match (self, other) {
            (Num(l), Num(r)) => l.cmp(r),
            (List(l), List(r)) => in_right_order_list(l, r),
            (Num(_), List(r)) => in_right_order_list(&[self], r),
            (List(l), Num(_)) => in_right_order_list(l, &[other]),
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse(s: &str) -> Packet {
    let (rest, packet) = packet(s).unwrap();
    debug_assert!(rest.is_empty());
    packet
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
    fn parser_test() {
        assert_eq!(Num(123), packet("123").unwrap().1);
        assert_eq!(List(vec![]), packet("[]").unwrap().1);
        assert_eq!(List(vec![Num(42)]), packet("[42]").unwrap().1);
        assert_eq!(
            List(vec![List(vec![]), Num(42)]),
            packet("[[],42]").unwrap().1
        );
    }
}
