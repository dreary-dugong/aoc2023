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

// seperate input into lines, find numbers, and find symbols
fn process(input: String) -> anyhow::Result<u32> {
    // offsets for finding neighbors of symbols
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
    // regex for finding symbols
    let symbol_pattern = Regex::new(r"[^\d\.\s]").unwrap();

    // Hash set to keep track of which positions neighbor symbols
    let mut symbol_neighbors = HashSet::new();

    let linecount = input.lines().count() as i32;

    // find symbols and record their neighbors
    for (lineno, line) in input.lines().enumerate() {
        for symbol_match in symbol_pattern.find_iter(line) {
            for offset in OFFSETS {
                let neighbor_line = lineno as i32 + offset.0;
                let neighbor_col = symbol_match.start() as i32 + offset.1;
                if 0 <= neighbor_line
                    && neighbor_line < linecount
                    && 0 <= neighbor_col
                    && neighbor_col < line.len() as i32
                {
                    symbol_neighbors.insert((neighbor_line as usize, neighbor_col as usize));
                }
            }
        }
    }

    let mut part_no_sum = 0;
    // for each number, check if it's by a symbol, and if so add to sum
    for (lineno, line) in input.lines().enumerate() {
        'match_loop: for num_match in num_pattern.find_iter(line) {
            for colno in num_match.range() {
                if symbol_neighbors.contains(&(lineno, colno)) {
                    part_no_sum += num_match.as_str().parse::<u32>().unwrap();
                    continue 'match_loop;
                }
            }
        }
    }

    Ok(part_no_sum)
}
