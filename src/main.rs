use anyhow::Result;
use memmap::MmapOptions;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author)]
struct Opt {
    #[structopt(short, long)]
    skip_verification: bool,

    #[structopt(short, long)]
    day_to_run: Option<usize>,

    #[structopt(short, long)]
    input_file: Option<PathBuf>,

    #[structopt(long)]
    skip_output: bool,
}

fn median(array: &[Duration]) -> Duration {
    if (array.len() % 2) == 0 {
        let ind_left = array.len() / 2 - 1;
        let ind_right = array.len() / 2;
        (array[ind_left] + array[ind_right]).div_f64(2.0)
    } else {
        array[(array.len() / 2)]
    }
}

fn d2s(d: Duration) -> String {
    format!("{:?}", d)
}

fn main() {
    let opt = Opt::from_args();
    let mut times = vec![];
    let mut times_io = vec![];

    let solutions: Vec<&dyn Fn(&str, bool, bool) -> Result<Duration>> = vec![
        &aoc22::day01::solve,
        &aoc22::day02::solve,
        /*
        &aoc22::day03::solve,
        &aoc22::day04::solve,
        &aoc22::day05::solve,
        &aoc22::day06::solve,
        &aoc22::day07::solve,
        &aoc22::day08::solve,
        &aoc22::day09::solve,
        &aoc22::day10::solve,
        &aoc22::day11::solve,
        &aoc22::day12::solve,
        &aoc22::day13::solve,
        &aoc22::day14::solve,
        &aoc22::day15::solve,
        &aoc22::day16::solve,
        &aoc22::day17::solve,
        &aoc22::day18::solve,
        &aoc22::day19::solve,
        &aoc22::day20::solve,
        &aoc22::day21::solve,
        &aoc22::day22::solve,
        &aoc22::day23::solve,
        &aoc22::day24::solve,
        &aoc22::day25::solve,
        */
    ];

    for (i, solution) in solutions.iter().enumerate() {
        if Some(i + 1) == opt.day_to_run || opt.day_to_run.is_none() {
            let input_file = match &opt.input_file {
                Some(path) => File::open(path).unwrap(),
                None => File::open(format!("inputs/day{:02}", i + 1)).unwrap(),
            };
            let mapped_input = unsafe { MmapOptions::new().map(&input_file).unwrap() };
            let input = std::str::from_utf8(&mapped_input).unwrap();

            let start = Instant::now();
            let t = match solution(input, !opt.skip_verification, !opt.skip_output) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Solution {i} failed: {e}");
                    continue;
                }
            };
            let solution_with_io = start.elapsed();
            println!(
                "Day {:02} took {:>14} to compute (with i/o: {:>14})",
                i + 1,
                d2s(t),
                d2s(solution_with_io)
            );
            times.push(t);
            times_io.push(solution_with_io);
        }
    }

    times.sort();
    times_io.sort();

    let total = times.iter().sum();
    let min = times.iter().min();
    let max = times.iter().max();

    let total_io = times_io.iter().sum();
    let min_io = times_io.iter().min();
    let max_io = times_io.iter().max();
    if opt.day_to_run.is_none() {
        println!(
            "\n         Total time for {} days: {:>14} (avg per day {:>10}, med: {:>10}, min: {:>10}, max: {:>10})",
            solutions.len(),
            d2s(total),
            d2s(total.div_f64(solutions.len() as f64)),
            d2s(median(&times)),
            d2s(*min.unwrap()),
            d2s(*max.unwrap()),
        );
        println!(
            "Total time with i/o for {} days: {:>14} (avg per day {:>10}, med: {:>10}, min: {:>10}, max: {:>10})",
            solutions.len(),
            d2s(total_io),
            d2s(total_io.div_f64(solutions.len() as f64)),
            d2s(median(&times_io)),
            d2s(*min_io.unwrap()),
            d2s(*max_io.unwrap()),
        );
    }
}
