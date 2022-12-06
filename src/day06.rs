use crate::U8Set;
use anyhow::Result;
use std::time::{Duration, Instant};
use std::collections::HashSet;

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input : Vec<_> = input.trim().chars().collect();

    let s = Instant::now();

    let LEN = 4;
    let part1 = input.windows(LEN).enumerate().find(|(i, w)| w.iter().collect::<HashSet<_>>().len() == LEN).unwrap().0 + LEN;
    dbg!(part1);

    let LEN = 14;
    let part2 = input.windows(LEN).enumerate().find(|(i, w)| w.iter().collect::<HashSet<_>>().len() == LEN).unwrap().0 + LEN;
    dbg!(part2);

    let e = s.elapsed();
    /*

    if verify_expected {
        assert_eq!(7917, part1);
        assert_eq!(2585, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    */
    Ok(e)
}
