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

    let (seeds, maps) = parse(input_string)?;
    let result = process(seeds, maps);

    println!("{}", result);

    Ok(())
}

/// given our input string, parse it into seeds and resource maps
fn parse(input: String) -> anyhow::Result<(Vec<Resource>, HashMap<String, ResourceMap>)> {
    let mut sections = input.split("\n\n");

    // feeling like a real rustacean now bb
    let seeds = sections
        .next()
        .ok_or(anyhow::anyhow!("input missing seed section"))?
        .split_whitespace()
        // skip seed label
        .skip(1)
        // I still don't know how to get results out of iterators
        .map(|seed_str| seed_str.parse::<u64>().unwrap())
        .map(|seed_id| Resource {
            name: "seed".to_string(),
            id: seed_id,
        })
        .collect::<Vec<_>>();

    // really gotta figure out throwing errors from closures
    let maps = sections
        .map(|section| {
            let mut lines = section.lines();

            // what resources are this map converting to/from
            let resource_map_label = lines.next().unwrap();
            let mut resource_name_iter =
                resource_map_label[..resource_map_label.len() - 1].split("-to-");
            let from = resource_name_iter.next().unwrap().to_string();
            let to = resource_name_iter
                .next()
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string();

            // convert remaining lines to ranges
            let ranges = lines
                .map(|line| {
                    let mut line_iter = line.split_whitespace();
                    let to_start = line_iter.next().unwrap().parse::<u64>().unwrap();
                    let from_start = line_iter.next().unwrap().parse::<u64>().unwrap();
                    let length = line_iter.next().unwrap().parse::<u64>().unwrap();

                    ResourceRange {
                        from_start,
                        to_start,
                        length,
                    }
                })
                .collect::<Vec<_>>();

            ResourceMap { from, to, ranges }
        })
        .map(|map| (map.from.to_string(), map))
        .collect::<HashMap<_, _>>();

    Ok((seeds, maps))
}

/// given our seeds and maps, convert our seeds all the way to locations
fn process(seeds: Vec<Resource>, maps: HashMap<String, ResourceMap>) -> u64 {
    seeds
        .into_iter()
        // convert seeds to locations
        .map(|seed| {
            let mut curr_resource = seed;
            while curr_resource.name != "location" {
                curr_resource = maps[&curr_resource.name]
                    .convert_resource(curr_resource)
                    .unwrap();
            }
            curr_resource
        })
        .map(|location| location.id)
        .min()
        .unwrap()
}

/// a resource that we're converting
struct Resource {
    name: String,
    id: u64,
}

/// a map for converting from one resource to another
struct ResourceMap {
    from: String,
    to: String,
    ranges: Vec<ResourceRange>,
}

impl ResourceMap {
    fn convert_resource(&self, resource: Resource) -> anyhow::Result<Resource> {
        if resource.name != self.from {
            panic!("called convert_resource on resource map and resource that are incompatible");
        }
        for range in &self.ranges {
            if range.can_convert(resource.id) {
                return Ok(Resource {
                    name: self.to.clone(),
                    id: range.convert_resource(resource.id),
                });
            }
        }

        Ok(Resource {
            name: self.to.clone(),
            id: resource.id,
        })
    }
}

/// an individual range in a resource map
struct ResourceRange {
    from_start: u64,
    to_start: u64,
    length: u64,
}
impl ResourceRange {
    /// given a resource range and a resource id, check if this range can convert it
    fn can_convert(&self, from_id: u64) -> bool {
        from_id >= self.from_start && from_id <= self.from_start + self.length
    }
    /// given a resource id that can be converted, return the resulting new resource id
    fn convert_resource(&self, from_id: u64) -> u64 {
        if !self.can_convert(from_id) {
            panic!("Called convert_resource on a range and resource that are incompatible")
        }
        from_id - self.from_start + self.to_start
    }
}
