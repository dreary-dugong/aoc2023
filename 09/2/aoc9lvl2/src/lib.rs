use std::fs;
use std::io;
use std::path::PathBuf;

extern crate clap;
use clap::Parser;

extern crate anyhow;

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

    let data = parse(input_string)?;
    let result = process(data);

    println!("{}", result);

    Ok(())
}

/// read input and parse into sequences of numbers
fn parse(input: String) -> anyhow::Result<Vec<Vec<i32>>> {
    Ok(input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|n| n.parse::<i32>().unwrap())
                .collect::<Vec<i32>>()
        })
        .collect())
}

/// find the sum of next numbers per sequence
fn process(data: Vec<Vec<i32>>) -> i32 {
    data.into_iter()
        .map(|sequence| {
            let mut firsts = Vec::new();
            let mut cur_seq = sequence;
            // make new sequences until we have one of all zeroes
            while cur_seq.iter().filter(|n| **n == 0).count() != cur_seq.len() {
                firsts.push(cur_seq[0]);
                cur_seq = cur_seq[1..]
                    .iter()
                    .zip(cur_seq[0..cur_seq.len() - 1].iter())
                    .map(|(l, s)| l - s)
                    .collect()
            }

            firsts
                .into_iter()
                .rev()
                .fold(0, |prediction, first| first - prediction)
        })
        .sum()
}
