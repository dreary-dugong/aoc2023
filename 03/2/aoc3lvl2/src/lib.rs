use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

extern crate clap;
use clap::Parser;

extern crate anyhow;

extern crate regex;
use regex::Regex;

#[derive(Parser, Debug)]
pub struct Args {
    /// path to the input file
    #[arg(short, long)]
    input: Option<PathBuf>,
}

enum InputConfig {
    File(PathBuf),
    Stdin,
}
pub struct Config {
    input: InputConfig,
}

impl Config {
    pub fn make() -> Self {
        let args = Args::parse();
        let input = if let Some(path) = args.input {
            InputConfig::File(path)
        } else {
            InputConfig::Stdin
        };

        Config { input }
    }
}

pub fn run(cfg: Config) -> anyhow::Result<()> {
    // figure out where to get our input from and read it into a string
    let input_string = match cfg.input {
        InputConfig::File(path) => fs::read_to_string(path)?,
        InputConfig::Stdin => {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            buf
        }
    };

    let result = process(input_string)?;

    println!("{}", result);

    Ok(())
}

// seperate input into lines, find numbers, and find gears
// then for each number, increment the count for any gears it's close to
// add up all gears
fn process(input: String) -> anyhow::Result<u32> {
    // offsets for finding neighbors of gears
    const OFFSETS: [(i32, i32); 8] = [
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (1, 0),
        (-1, 1),
        (-1, -1),
        (-1, 0),
    ];
    // regex for finding numbers
    let num_pattern = Regex::new(r"\d+").unwrap();
    // regex for finding potential gears
    let pot_gear_pattern = Regex::new(r"\*").unwrap();

    // list of potential gears
    let mut potential_gears = Vec::new();

    let linecount = input.lines().count() as i32;

    // find gears and record their neighbors
    for (lineno, line) in input.lines().enumerate() {
        for gear_match in pot_gear_pattern.find_iter(line) {
            let mut neighbors = HashSet::new();
            for offset in OFFSETS {
                let neighbor_line = lineno as i32 + offset.0;
                let neighbor_col = gear_match.start() as i32 + offset.1;
                if 0 <= neighbor_line
                    && neighbor_line < linecount
                    && 0 <= neighbor_col
                    && neighbor_col < line.len() as i32
                {
                    neighbors.insert((neighbor_line as usize, neighbor_col as usize));
                }
            }

            potential_gears.push(PotentialGear {
                neighbors,
                neighboring_nums: 0,
                ratio: 1,
            });
        }
    }

    // for each number, check if it's by a gear
    // if it is, increment that gear's counters
    for (lineno, line) in input.lines().enumerate() {
        let num_matches = num_pattern.find_iter(line).collect::<Vec<_>>();
        'gear_loop: for pot_gear in potential_gears.iter_mut() {
            for num_match in num_matches.iter() {
                for colno in num_match.range() {
                    if pot_gear.neighbors.contains(&(lineno, colno)) {
                        pot_gear.neighboring_nums += 1;
                        if pot_gear.neighboring_nums < 3 {
                            pot_gear.ratio *= num_match.as_str().parse::<u32>().unwrap();
                        }
                        // don't double count the same gear for each digit
                        continue 'gear_loop
                    }
                }
            }
        }
    }

    Ok(potential_gears.into_iter().filter(|gear| gear.neighboring_nums == 2).map(|gear| gear.ratio).sum())
}

#[derive(Debug)]
struct PotentialGear {
    neighbors: HashSet<(usize, usize)>,
    neighboring_nums: u32,
    ratio: u32,
}
