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

    let data = parse(&input_string)?;
    let result = process(data);

    println!("{}", result);

    Ok(())
}

/// given our input, seperate it into cards with winning numbers and numbers we have
fn parse(input: &str) -> anyhow::Result<Vec<Card>> {
    Ok(input
        // split into lines
        .lines()
        // remove card identifier info
        .map(|line| &line[line.find(':').unwrap() + 1..])
        // split into winning numbers and have numbers
        .map(|line| line.split('|'))
        // split up numbes and put them on a card
        .map(|mut nums| {
            let winning_nums = nums.next().unwrap().split_whitespace().collect::<Vec<_>>();
            let have_nums = nums.next().unwrap().split_whitespace().collect::<Vec<_>>();
            Card {
                winning_nums,
                have_nums,
            }
        })
        .collect())
}

/// given a vector of cards, calculate the total score
fn process(cards: Vec<Card>) -> u32 {
    let mut total_score = 0;
    for card in cards {
        let mut matching_nums = 0;
        for hn in card.have_nums {
            if card.winning_nums.contains(&hn) {
                matching_nums += 1;
            }
        }
        total_score += if matching_nums == 0 {
            0
        } else {
            1 << (matching_nums - 1)
        };
    }
    total_score
}

struct Card<'a> {
    winning_nums: Vec<&'a str>,
    have_nums: Vec<&'a str>,
}
