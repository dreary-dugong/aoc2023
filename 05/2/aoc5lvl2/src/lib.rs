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
fn parse(input: String) -> anyhow::Result<(ResourceCollection, HashMap<String, ResourceMap>)> {
    let mut sections = input.split("\n\n");

    // get rid of label and only take numbers
    let seed_range_caps = sections
        .next()
        .ok_or(anyhow::anyhow!("input missing seed section"))?
        .split_whitespace()
        // skip seed label
        .skip(1)
        .collect::<Vec<_>>();

    // get our inital resource ranges from the seeds
    let seed_ranges = seed_range_caps
        // split into ranges
        .chunks(2)
        // convert to numbers convert to ranges
        .map(|pair| {
            let start = pair[0].parse::<u64>().unwrap();
            let length = pair[1].parse::<u64>().unwrap();
            let end = start + length - 1; // inclusive
            ResourceRange { start, end }
        })
        .collect::<Vec<_>>();

    let seed_collection = ResourceCollection {
        name: String::from("seed"),
        ranges: seed_ranges,
    };

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

            // convert remaining lines to ranges mappers
            let mappers = lines
                .map(|line| {
                    let mut line_iter = line.split_whitespace();
                    let to_start = line_iter.next().unwrap().parse::<u64>().unwrap();
                    let from_start = line_iter.next().unwrap().parse::<u64>().unwrap();
                    let length = line_iter.next().unwrap().parse::<u64>().unwrap();

                    ResourceMapRange {
                        from_start,
                        to_start,
                        length,
                    }
                })
                .collect::<Vec<_>>();

            ResourceMap { from, to, mappers }
        })
        .map(|map| (map.from.to_string(), map))
        .collect::<HashMap<_, _>>();

    Ok((seed_collection, maps))
}

/// given our seeds and maps, convert our seeds all the way to locations
fn process(seeds: ResourceCollection, maps: HashMap<String, ResourceMap>) -> u64 {
    let mut cur_collection = seeds;
    while cur_collection.name != "location" {
        let cur_target = &cur_collection.name;
        cur_collection = maps[cur_target].convert_resource_collection(cur_collection);
    }

    // the smallest number of every range must be the start, so just compare those
    cur_collection
        .ranges
        .into_iter()
        .map(|location_range| location_range.start)
        .min()
        .unwrap()
}

/// a resource that we're converting
struct ResourceRange {
    // start and end are inclusive
    start: u64,
    end: u64,
}

/// a collection of resources of the same type represented by ranges
struct ResourceCollection {
    name: String,
    ranges: Vec<ResourceRange>,
}

/// a map for converting from one resource to another
struct ResourceMap {
    from: String,
    to: String,
    mappers: Vec<ResourceMapRange>,
}

impl ResourceMap {
    // given a vector of resource ranges of the 'from' type, convert them all to ranges of the 'to' type
    fn convert_resource_collection(&self, r_collection: ResourceCollection) -> ResourceCollection {
        if r_collection.name != self.from {
            panic!("attempt to use resourcemap on a resourcecollection it can't convert");
        }
        // ranges that have been converted to the new type
        let mut converted_ranges = Vec::new();
        // ranges that are yet to be converted
        let mut conversion_queue = r_collection.ranges;
        // go through our queue until all's been converted
        'range_loop: while let Some(cur_range) = conversion_queue.pop() {
            // for each range, check all our mappers to see if they apply
            for mapper in &self.mappers {
                if mapper.has_overlap(&cur_range) {
                    let (converted_range, overflow_ranges) =
                        mapper.convert_resource_range(cur_range);
                    converted_ranges.push(converted_range);
                    conversion_queue.extend(overflow_ranges);
                    continue 'range_loop;
                }
            }
            // if no mapper applied, convert directly
            converted_ranges.push(ResourceRange {
                start: cur_range.start,
                end: cur_range.end,
            });
        }

        // during the above process, we fracture the ranges. It's possible that after conversion,
        // some converted ranges may be able to be combined (e.g. if one's start is the same as another's end).
        // This would of course save memory and we know memory is our bottle neck for the brute force solution
        // if needed, we can attempt to combine all like ranges before returning them
        // that's super annoying though and my pc has 32gb so I *think* we can get away without it
        // premature optimization and all that jazz
        // update: we got away without it but I'm keeping the comment so I look smart
        ResourceCollection {
            name: self.to.clone(),
            ranges: converted_ranges,
        }
    }
}

/// an individual range in a resource map
struct ResourceMapRange {
    from_start: u64,
    to_start: u64,
    length: u64,
}
impl ResourceMapRange {
    /// given a resource range, check if it has overlap with this mapping (aka if this mapping needs to be used on it)
    fn has_overlap(&self, r_range: &ResourceRange) -> bool {
        let from_end = self.from_start + self.length - 1;
        (self.from_start <= r_range.start && r_range.start <= from_end)
            || (self.from_start <= r_range.end && r_range.end <= from_end)
    }

    /// given a resource range with overlap, convert it into one resource range of the new type and 0, 1, or 2 smaller ranges of the original type
    fn convert_resource_range(
        &self,
        r_range: ResourceRange,
    ) -> (ResourceRange, Vec<ResourceRange>) {
        if !self.has_overlap(&r_range) {
            panic!("Called convert_resource_range on a maprange and resourcerange that are incompatible")
        }
        let mut old_type_ranges = Vec::new();
        // inclusive
        let from_end = self.from_start + self.length - 1;

        // start of the range we can convert to the new type
        let convertible_start;
        // check if there's overflow at the start of the resource range that we can't convert (if there is, it becomes an old type range)
        if self.from_start > r_range.start {
            convertible_start = self.from_start;
            old_type_ranges.push(ResourceRange {
                start: r_range.start,
                end: self.from_start - 1,
            });
        } else {
            convertible_start = r_range.start;
        }

        // check if there's overflow at the end of the resource range we can't convert (if there is, it becomes an old type range)
        let convertible_end;
        if from_end < r_range.end {
            convertible_end = from_end;
            old_type_ranges.push(ResourceRange {
                start: from_end + 1,
                end: r_range.end,
            });
        } else {
            convertible_end = r_range.end;
        }

        // convert what we can to a new type range
        let converted_range = ResourceRange {
            start: self.to_start + convertible_start - self.from_start,
            end: self.to_start + convertible_end - self.from_start,
        };

        (converted_range, old_type_ranges)
    }
}
