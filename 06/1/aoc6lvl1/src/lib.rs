use std::fs;
use std::io;
use std::iter;
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

/// parse the input data into races
fn parse(input: String) -> anyhow::Result<Vec<Race>> {
    // seperate lines
    let mut lines_iter = input.lines();
    let time_line = lines_iter
        .next()
        .ok_or(anyhow::anyhow!("Missing first line in file"))?;
    let dist_line = lines_iter
        .next()
        .ok_or(anyhow::anyhow!("Missing second line in file"))?;

    // associate values and make race structs
    // man I still cannot figure out how to throw errors from closures
    Ok(
        iter::zip(time_line.split_whitespace(), dist_line.split_whitespace())
            .skip(1)
            .map(|(t, d)| Race {
                time: t.parse::<f32>().expect("failed to parse number"),
                distance: d.parse::<f32>().expect("failed to parse number"),
            })
            .collect(),
    )
}

fn process(races: Vec<Race>) -> u32 {
    races
        .into_iter()
        .map(|race| {
            // we're using the quadratic formula to find the two roots, then counting the integers between
            let sqrt_term = (race.time * race.time - 4_f32 * race.distance).sqrt();
            let b_term = -1_f32 * race.time;
            let a_term = -2_f32;

            let lower_root = (b_term + sqrt_term) / a_term;
            let upper_root = (b_term - sqrt_term) / a_term;

            (upper_root.ceil() - (lower_root.floor() + 1_f32)) as u32
        })
        .product()
}

struct Race {
    time: f32,
    distance: f32,
}
