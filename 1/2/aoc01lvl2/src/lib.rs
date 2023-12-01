use std::fs;
use std::io;
use std::path::PathBuf;

extern crate clap;
use clap::Parser;

extern crate anyhow;

extern crate onig;
use onig::Regex;

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

fn parse(input: String) -> anyhow::Result<Vec<String>> {
    Ok(input.lines().map(|l| l.to_string()).collect())
}

fn process(data: Vec<String>) -> u32 {
    data.into_iter()
        .map(|line| {
            let pattern =
                Regex::new("(?=([0-9]|one|two|three|four|five|six|seven|eight|nine))").unwrap();
            for ch in pattern.find_iter(&line) {
                println!("{:?}", ch);
            }
            let captures = pattern.captures_iter(&line).collect::<Vec<_>>();
            captures.iter().for_each(|c| println!("{:?}", c.at(0)));

            let first = captures[0].at(0).unwrap();
            let last = captures[captures.len() - 1].at(0).unwrap();
            println!("{}, {}, {}", line, first, last);
            convert_match_to_digit(first) * 10 + convert_match_to_digit(last)
        })
        .sum()
}

fn convert_match_to_digit(m: &str) -> u32 {
    match m {
        "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        other => other.parse().unwrap(),
    }
}
