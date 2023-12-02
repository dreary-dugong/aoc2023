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

/// parse game strings into games containing samples of optional color counts
fn parse(input: String) -> anyhow::Result<Vec<Game>> {
    let mut games = Vec::new();

    for line in input.lines() {
        let game_label_end = line.find(':').expect("invalid input, missing colon");
        let id = line[5..game_label_end].parse::<u32>()?;

        let mut cur_game = Game {
            id,
            samples: Vec::new(),
        };

        for sample_str in line[game_label_end + 2..].split("; ") {
            let (mut red, mut green, mut blue) = (None, None, None);
            for cube_sample in sample_str.split(", ") {
                let mut iter = cube_sample.split(' ');
                let count = iter
                    .next()
                    .expect("missing cube count in sample")
                    .parse::<u32>()
                    .expect("bad cube count");
                let color = iter.next().expect("missing cube color in sample");

                match color {
                    "red" => red = Some(count),
                    "green" => green = Some(count),
                    "blue" => blue = Some(count),
                    _ => panic!("bad cube color in sample"),
                };

                cur_game.samples.push(Sample { red, green, blue });
            }
        }

        games.push(cur_game);
    }

    Ok(games)
}

/// process game structs to get our answer
fn process(games: Vec<Game>) -> u32 {
    let mut total_power = 0;
    for game in games {
        let (mut min_red, mut min_green, mut min_blue) = (0, 0, 0);
        for sample in game.samples {
            if let Some(red) = sample.red {
                min_red = min_red.max(red);
            }
            if let Some(green) = sample.green {
                min_green = min_green.max(green);
            }
            if let Some(blue) = sample.blue {
                min_blue = min_blue.max(blue);
            }
        }

        total_power += min_red * min_green * min_blue;
    }

    total_power
}

#[derive(Debug)]
struct Game {
    id: u32,
    samples: Vec<Sample>,
}

#[derive(Debug)]
struct Sample {
    red: Option<u32>,
    green: Option<u32>,
    blue: Option<u32>,
}
