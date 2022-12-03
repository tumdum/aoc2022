use anyhow::Result;
use std::time::{Duration, Instant};

#[derive(Default, PartialEq, Debug)]
struct U8Set {
    data: [u64; 4],
}

impl U8Set {
    fn contains(&self, value: u8) -> bool {
        let idx = (value / 64) as usize;
        let off = 1 << (value % 64);
        let ret = self.data[idx] & off;
        ret != 0
    }

    fn insert(&mut self, value: u8) -> bool {
        let idx = (value / 64) as usize;
        let off = 1 << (value % 64);
        let ret = self.data[idx] & off;
        self.data[idx] |= off;
        ret != 0
    }

    fn intersection(&self, other: &Self) -> Self {
        Self {
            data: [
                self.data[0] & other.data[0],
                self.data[1] & other.data[1],
                self.data[2] & other.data[2],
                self.data[3] & other.data[3],
            ],
        }
    }

    fn iter(&'_ self) -> impl Iterator<Item = u8> + '_ {
        (0u8..=255).filter(|i| self.contains(*i))
    }
}

impl FromIterator<u8> for U8Set {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        let mut ret = Self::default();
        for i in iter {
            ret.insert(i);
        }
        ret
    }
}

fn score(b: u8) -> u64 {
    if b < b'a' {
        let b = b - b'A' + 1;
        b as u64 + 26
    } else {
        let b = b - b'a' + 1;
        b as u64
    }
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<Vec<u8>> = input.lines().map(|l| l.bytes().collect()).collect();

    let s = Instant::now();

    let part1: u64 = input
        .iter()
        .map(|v| {
            let len = v.len() / 2;
            let l: U8Set = v[..len].iter().copied().collect();
            let r: U8Set = v[len..].iter().copied().collect();
            (l, r)
        })
        .flat_map(|(l, r)| l.intersection(&r).iter().next())
        .map(score)
        .sum();

    let part2: u64 = input
        .chunks(3)
        .flat_map(|c| {
            let a: U8Set = c[0].iter().copied().collect();
            let b: U8Set = c[1].iter().copied().collect();
            let c: U8Set = c[2].iter().copied().collect();
            a.intersection(&b).intersection(&c).iter().next()
        })
        .map(score)
        .sum();

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(7917, part1);
        assert_eq!(2585, part2);
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
    fn score_test() {
        assert_eq!(score(b'a'), 1);
        assert_eq!(score(b'z'), 26);
        assert_eq!(score(b'A'), 27);
        assert_eq!(score(b'Z'), 52);
    }

    #[test]
    fn empty() {
        let s = U8Set::default();
        for i in 0u8..=255 {
            assert!(!s.contains(i));
        }
    }

    #[test]
    fn insert() {
        for i in 0u8..=255 {
            let mut s = U8Set::default();
            assert!(!s.contains(i));
            assert!(!s.insert(i));
            assert!(s.contains(i));
            for j in 0u8..=255 {
                if j != i {
                    assert!(!s.contains(j));
                }
            }
        }
    }

    #[test]
    fn from_iter() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        for i in 0u8..=255 {
            if v.contains(&i) {
                assert!(s.contains(i));
            } else {
                assert!(!s.contains(i));
            }
        }
    }

    #[test]
    fn iter() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        let v2: Vec<_> = s.iter().collect();
        assert_eq!(v, v2);
    }

    #[test]
    fn intersection_self() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        let si = s.intersection(&s);
        assert_eq!(si, s);
    }

    #[test]
    fn intersection_sub() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        let sub = vec![1u8, 68, 99, 157, 200, 255];
        let s_sub: U8Set = v.clone().into_iter().collect();
        assert_eq!(s_sub, s.intersection(&s_sub));
        assert_eq!(s_sub, s_sub.intersection(&s));
    }

    #[test]
    fn intersection() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        let w = vec![1u8, 15, 68, 99, 121, 157, 200, 254, 255];
        let s_sub: U8Set = v.clone().into_iter().collect();
        let i = vec![1u8, 68, 99, 157, 200, 255];
        let si: U8Set = v.clone().into_iter().collect();
        assert_eq!(si, s.intersection(&s_sub));
        assert_eq!(si, s_sub.intersection(&s));
    }
}
