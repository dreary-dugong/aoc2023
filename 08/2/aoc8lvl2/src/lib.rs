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

fn process(directions: Vec<Direction>, graph: HashMap<String, Node>) -> u64 {
    // start at every node that ends with A
    let start_nodes: Vec<&Node> = graph
        .values()
        .filter(|node| node.label.ends_with('A'))
        .collect();

    // for each A node, figure out which cycles will lead them to repeat Z
    let mut cycle_lists = Vec::new();
    for node in start_nodes.into_iter() {
        let mut zs = Vec::new(); // the z nodes we've encountered so far
        let mut ks = Vec::new(); // the constants associated with each Z cycle
        let mut cycles = Vec::new(); // the cycles we've completed so far
        let mut direction_iter = directions.iter().cycle();

        // continue until we've found at least one z cycle and have found the length for every z cycle
        let mut cur_node = node;
        let mut num_steps = 0;
        while zs.is_empty() || zs.len() != cycles.len() {
            num_steps += 1;
            let direction = direction_iter.next().unwrap();
            let temp = match direction {
                Direction::Left => graph.get(&cur_node.left).unwrap(),
                Direction::Right => graph.get(&cur_node.right).unwrap(),
            };
            cur_node = temp;
            if cur_node.label.ends_with('Z') {
                if zs.contains(&cur_node) {
                    let k = ks[zs.iter().position(|z| z == &cur_node).unwrap()];
                    cycles.push(ZCycle {
                        constant: k,
                        length: num_steps - k,
                    });
                } else {
                    zs.push(cur_node);
                    ks.push(num_steps);
                }
            }
        }
        cycle_lists.push(cycles);
    }

    println!("Constructed all cycle lists.");

    // find the cycle list we'll use to track down the number of steps
    let mut best_cycle_list = &cycle_lists[0];
    let mut max_power = get_power(best_cycle_list);
    for cycle_list in &cycle_lists {
        // we want a cycle list that will let us skip a bunch of steps every iteration,
        // so we find the one that lets us skip the most on average
        let cur_power = get_power(cycle_list);
        if cur_power > max_power {
            max_power = cur_power;
            best_cycle_list = cycle_list;
        }
    }
    println!(
        "Identified best cycle list; length: {}, power: {}",
        best_cycle_list.len(),
        max_power
    );

    // here's the fun part. we know that the solution is, for each cycle, k + rl where k is the cycle's constant, l is the cycle's length,
    // and r is the number of times that cycle has been repeated. For any value s, we can quickly check if s satisfies at least one z cycle per starting node
    // So, we continually try s by taking our biggest cycle (or combination of cycles for a single node) and repeating it over and over and checking each time
    let mut found_solution = false;
    let mut num_cycle_repeats = 0;
    while !found_solution {
        if num_cycle_repeats % 10000000 == 0 {
            println!("trying with {} cycle repeats", num_cycle_repeats);
        }
        'cycle_loop: for cycle in best_cycle_list {
            let steps_attempt = cycle.constant + cycle.length * num_cycle_repeats;
            'verify_loop: for cycle_list in &cycle_lists {
                for cycle in cycle_list {
                    if cycle.is_valid_steps(steps_attempt) {
                        continue 'verify_loop;
                    }
                }
                continue 'cycle_loop;
            }
            found_solution = true;
            return steps_attempt;
        }
        num_cycle_repeats += 1;
    }

    unreachable!("we either find a solution or loop forever.");
}

// given a cycle list, return the average number of steps it will skip per iteration
fn get_power(cycle_list: &Vec<ZCycle>) -> u64 {
    cycle_list
        .iter()
        .map(|cycle| cycle.length)
        .fold(0, |acc, l| acc + l)
        / (cycle_list.len() as u64)
}

struct ZCycle {
    // the number of steps necessary to enter the cycle
    constant: u64,
    // the length of the cycle
    length: u64,
}

impl ZCycle {
    // after a given number of steps, determine whether this cycle will land on Z
    fn is_valid_steps(&self, steps: u64) -> bool {
        (steps - self.constant) % self.length == 0
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Node {
    label: String,
    left: String,
    right: String,
}
