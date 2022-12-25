use anyhow::Result;
use std::time::{Duration, Instant};

fn c2d(c: char) -> i64 {
    match c {
        '2' => 2,
        '1' => 1,
        '0' => 0,
        '-' => -1,
        '=' => -2,
        _ => todo!(),
    }
}

fn d2c(i: i64) -> char {
    match i {
        2 => '2',
        1 => '1',
        0 => '0',
        -1 => '-',
        -2 => '=',
        _ => todo!("{}", i),
    }
}

fn snafu2dec(s: &str) -> i64 {
    let mut tmp: Vec<_> = s.chars().map(c2d).collect();
    tmp.reverse();
    snafu2dec2(&tmp)
}

fn snafu2dec2(s: &[i64]) -> i64 {
    let mut ret = 0;
    let mut base = 1;
    for d in s {
        ret += d * base;
        base *= 5;
    }
    ret
}

fn dec2snafu(mut i: i64, boundries: &[(i64, i64)]) -> String {
    let mut ret = String::new();
    let mut c = boundries.len() - 1;
    loop {
        if boundries[c].0 < i {
            break;
        }
        c -= 1;
    }
    c += 1;
    loop {
        let mut digit = 0;
        if c > 0 {
            while i > 0 && i.abs() > boundries[c - 1].0 {
                digit += 1;
                i -= boundries[c].1;
            }
            while i < 0 && i.abs() > boundries[c - 1].0 {
                digit -= 1;
                i += boundries[c].1;
            }
        } else {
            digit = i / boundries[c].1;
        }
        ret.push(d2c(digit));
        if c == 0 {
            break;
        }
        c -= 1;
    }
    ret
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: i64 = input.lines().map(snafu2dec).sum();

    let s = Instant::now();

    let mut boundries = vec![];
    let mut base = 1;
    let mut sum = 0;
    for _ in 0..25 {
        let next = sum + 2 * base;
        boundries.push((next, base));
        sum = next;
        base *= 5;
    }

    assert_eq!(dec2snafu(107, &boundries), "1-12");
    assert_eq!(dec2snafu(198, &boundries), "2=0=");
    assert_eq!(dec2snafu(12345, &boundries), "1-0---0");
    assert_eq!(dec2snafu(314159265, &boundries), "1121-1110-1=0");

    let part1 = dec2snafu(input, &boundries);

    let e = s.elapsed();

    if verify_expected {
        assert_eq!("2-==10===-12=2-1=-=0", part1);
    }
    if output {
        println!("\t{}", part1);
    }
    Ok(e)
}
