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

/// parse the input data into a single race
fn parse(input: String) -> anyhow::Result<Race> {
    // seperate lines
    let mut lines_iter = input.lines();

    // for each line, skip the label and shove the characters together
    let time = lines_iter
        .next()
        .ok_or(anyhow::anyhow!("Missing first line in file"))?
        .split_whitespace()
        .skip(1)
        .fold(String::new(), |mut acc, s| {
            acc.push_str(s);
            acc
        })
        .parse::<f64>()?;
    let distance = lines_iter
        .next()
        .ok_or(anyhow::anyhow!("Missing second line in file"))?
        .split_whitespace()
        .skip(1)
        .fold(String::new(), |mut acc, s| {
            acc.push_str(s);
            acc
        })
        .parse::<f64>()?;

    Ok(Race { time, distance })
}

fn process(race: Race) -> u32 {
    // we're using the quadratic formula to find the two roots, then counting the integers between
    let sqrt_term = (race.time * race.time - 4_f64 * race.distance).sqrt();
    let b_term = -1_f64 * race.time;
    let a_term = -2_f64;

    let lower_root = (b_term + sqrt_term) / a_term;
    let upper_root = (b_term - sqrt_term) / a_term;

    (upper_root.ceil() - (lower_root.floor() + 1_f64)) as u32
}

struct Race {
    time: f64,
    distance: f64,
}
