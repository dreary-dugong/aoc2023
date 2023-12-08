use std::collections::HashMap;
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

    let (directions, graph) = parse(input_string)?;
    let result = process(directions, graph);

    println!("{}", result);

    Ok(())
}

/// parse input into a vector of directions and a hashmap associating labels with nodes
fn parse(input: String) -> anyhow::Result<(Vec<Direction>, HashMap<String, Node>)> {
    // graphs in rust are hard but this one isn't awful yet
    let mut line_iter = input.lines();
    // parse directions
    let direction_line = line_iter
        .next()
        .ok_or(anyhow::anyhow!("empty input file"))?;
    let directions = direction_line
        .chars()
        .map(|c| match c {
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => panic!(
                "invalid character in direction list; I still can't return errors from closures"
            ),
        })
        .collect::<Vec<Direction>>();

    // skip blank line
    line_iter
        .next()
        .ok_or(anyhow::anyhow!("missing line after input line"))?;

    // contstruct graph by taking character indices
    let graph = line_iter
        .map(|line| {
            let label = line[0..=2].to_string();
            let left = line[7..=9].to_string();
            let right = line[12..=14].to_string();
            Node { label, left, right }
        })
        .map(|node| (node.label.clone(), node))
        .collect::<HashMap<String, Node>>();

    Ok((directions, graph))
}

fn process(directions: Vec<Direction>, graph: HashMap<String, Node>) -> u32 {
    let mut cur_node = graph.get("AAA").expect("invalid graph");
    let mut direction_iter = directions.into_iter().cycle();
    let mut num_steps = 0;
    // traverse graph until we find our target, counting steps
    while cur_node.label != "ZZZ" {
        cur_node = match direction_iter.next().unwrap() {
            Direction::Left => graph.get(&cur_node.left).expect("Missing node in graph"),
            Direction::Right => graph.get(&cur_node.right).expect("Missing node in graph"),
        };
        num_steps += 1;
    }

    num_steps
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
}
#[derive(Debug)]
struct Node {
    label: String,
    left: String,
    right: String,
}
