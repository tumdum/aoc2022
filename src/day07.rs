use anyhow::Result;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
enum Entry {
    File(String, usize),
    Dir(String),
}

#[derive(Debug)]
enum Cmd {
    Ls(Vec<Entry>),
    Cd(String),
}

#[derive(Default, Debug)]
struct Command {
    cmd: String,
    output: Vec<String>,
}

pub fn solve(input: &str, verify_expected: bool, output: bool) -> Result<Duration> {
    let input: Vec<&str> = input.lines().collect();
    let mut commands: Vec<Command> = vec![];
    let mut next = Command::default();
    for line in input {
        if line.starts_with('$') {
            if !next.cmd.is_empty() {
                commands.push(next);
            }
            next = Command {
                cmd: line.to_owned(),
                output: vec![],
            };
        } else {
            next.output.push(line.to_owned());
        }
    }
    commands.push(next);

    let s = Instant::now();

    let commands: Vec<Cmd> = commands
        .into_iter()
        .map(|c| {
            if c.cmd == "$ ls" {
                let output = c
                    .output
                    .into_iter()
                    .map(|l| {
                        let mut l = l.split(' ');
                        let size_str = l.next().unwrap();
                        let name = l.next().unwrap();
                        if size_str == "dir" {
                            Entry::Dir(name.to_owned())
                        } else {
                            Entry::File(name.to_owned(), size_str.parse().unwrap())
                        }
                    })
                    .collect();
                Cmd::Ls(output)
            } else {
                let mut cmd = c.cmd.split(' ').skip(2);
                Cmd::Cd(cmd.next().unwrap().to_owned())
            }
        })
        .collect();

    let mut dir2files: HashMap<Vec<String>, Vec<Entry>> = HashMap::default();
    let mut cwd = vec!["/".to_owned()];
    for c in commands {
        match c {
            Cmd::Cd(s) if s == "/" => {
                cwd = vec![s.clone()];
            }
            Cmd::Cd(s) if s == ".." => {
                cwd.pop();
            }
            Cmd::Cd(s) => {
                cwd.push(s);
            }
            Cmd::Ls(entries) => {
                for e in entries {
                    dir2files.entry(cwd.clone()).or_default().push(e);
                }
            }
        }
    }
    let mut dir2files: Vec<(String, Vec<Entry>)> = dir2files
        .into_iter()
        .map(|(name, content)| (name.join("/"), content))
        .collect();

    dir2files.sort_by_key(|(name, _)| Reverse(name.len()));

    let mut total_sizes: HashMap<String, usize> = HashMap::default();
    for (name, entries) in dir2files {
        let mut total = 0;
        for e in entries {
            match e {
                Entry::File(_, size) => total += size,
                Entry::Dir(d) => {
                    let name = format!("{name}/{d}");
                    total += total_sizes.get(&name).unwrap_or(&0);
                }
            }
        }
        total_sizes.insert(name, total);
    }

    let part1: usize = total_sizes
        .iter()
        .map(|(_, size)| size)
        .filter(|s| **s <= 100000)
        .sum();

    let total = 70000000;
    let need = 30000000;
    let current_unused = total - *total_sizes.get("/").unwrap();
    let to_delete = need - current_unused;

    let mut total_sizes: Vec<(String, usize)> = total_sizes.into_iter().collect();
    total_sizes.sort_by_key(|(_, size)| *size);
    let idx = match total_sizes.binary_search_by_key(&to_delete, |(_, size)| *size) {
        Ok(i) => i,
        Err(i) => i,
    };
    let part2 = total_sizes[idx].1;

    let e = s.elapsed();

    if verify_expected {
        assert_eq!(1423358, part1);
        assert_eq!(545729, part2);
    }
    if output {
        println!("\t{}", part1);
        println!("\t{}", part2);
    }
    Ok(e)
}
