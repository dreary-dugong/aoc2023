use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
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

/// parse hands from input
fn parse(input: String) -> anyhow::Result<Vec<Hand>> {
    Ok(input
        .lines()
        .map(|line| {
            let mut line_iter = line.split_whitespace();
            let cards = line_iter
                .next()
                .expect("missing cards")
                .chars()
                .collect::<Vec<char>>();
            let bid = line_iter
                .next()
                .expect("missing bid")
                .parse::<u32>()
                .expect("failed to parse bid");
            Hand { cards, bid }
        })
        .collect::<Vec<_>>())
}

/// just sort our hands and assign scores
fn process(mut hands: Vec<Hand>) -> u32 {
    hands.sort_unstable();
    hands
        .into_iter()
        .enumerate()
        .fold(0, |acc, (i, hand)| acc + hand.bid * (i + 1) as u32)
}

/// return the relative strength of a card
fn get_card_strength(card: &char) -> usize {
    match card {
        'A' => 12,
        'K' => 11,
        'Q' => 10,
        'J' => 9,
        'T' => 8,
        '9' => 7,
        '8' => 6,
        '7' => 5,
        '6' => 4,
        '5' => 3,
        '4' => 2,
        '3' => 1,
        '2' => 0,
        _ => panic!("called get_card_strength on invalid card character"),
    }
}
#[derive(PartialEq, Eq)]
/// represent the type of a hand, to be used for scoring
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}
impl PartialOrd for HandType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandType {
    /// compare two hand types and see which scores higher
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        match (self, other) {
            (Self::FiveOfAKind, _) => Ordering::Greater,
            (_, Self::FiveOfAKind) => Ordering::Less,
            (Self::FourOfAKind, _) => Ordering::Greater,
            (_, Self::FourOfAKind) => Ordering::Less,
            (Self::FullHouse, _) => Ordering::Greater,
            (_, Self::FullHouse) => Ordering::Less,
            (Self::ThreeOfAKind, _) => Ordering::Greater,
            (_, Self::ThreeOfAKind) => Ordering::Less,
            (Self::TwoPair, _) => Ordering::Greater,
            (_, Self::TwoPair) => Ordering::Less,
            (Self::OnePair, _) => Ordering::Greater,
            (_, Self::OnePair) => Ordering::Less,
            (_, _) => unreachable!("all cases exhausted (we check equals above)"),
        }
    }
}

struct Hand {
    cards: Vec<char>,
    bid: u32,
}

// methods necessary to implement the traits for sorting
impl Hand {
    /// figure out the hand's type by checking each
    fn get_type(&self) -> HandType {
        if self.is_five_of_a_kind() {
            HandType::FiveOfAKind
        } else if self.is_four_of_a_kind() {
            HandType::FourOfAKind
        } else if self.is_full_house() {
            HandType::FullHouse
        } else if self.is_three_of_a_kind() {
            HandType::ThreeOfAKind
        } else if self.is_two_pair() {
            HandType::TwoPair
        } else if self.is_one_pair() {
            HandType::OnePair
        } else {
            HandType::HighCard
        }
    }
    /// return an array countaing the counts of each card type in a hand
    fn get_card_counts(&self) -> [u32; 13] {
        self.cards.iter().fold([0; 13], |mut acc, card| {
            acc[get_card_strength(card)] += 1;
            acc
        })
    }
    // verify if a hand is a given type
    fn is_five_of_a_kind(&self) -> bool {
        self.get_card_counts().contains(&5)
    }
    fn is_four_of_a_kind(&self) -> bool {
        self.get_card_counts().contains(&4)
    }
    fn is_full_house(&self) -> bool {
        let counts = self.get_card_counts();
        counts.contains(&3) && counts.contains(&2)
    }
    fn is_three_of_a_kind(&self) -> bool {
        !self.is_full_house() && self.get_card_counts().contains(&3)
    }
    fn is_two_pair(&self) -> bool {
        !self.is_full_house()
            && !self.is_three_of_a_kind()
            && self
                .get_card_counts()
                .into_iter()
                .filter(|n| *n == 2)
                .count()
                == 2
    }
    fn is_one_pair(&self) -> bool {
        !self.is_full_house()
            && !self.is_three_of_a_kind()
            && !self.is_two_pair()
            && self
                .get_card_counts()
                .into_iter()
                .filter(|n| *n == 2)
                .count()
                == 1
    }
}

impl PartialEq for Hand {
    /// compare 2 hands and decide if they're equal
    fn eq(&self, other: &Self) -> bool {
        // if the type isn't equal, they're not equal
        if self.get_type() != other.get_type() {
            return false;
        } else {
            // if the type is equal, check if the cards are equal
            for (selfval, otherval) in iter::zip(self.cards.iter(), other.cards.iter()) {
                if selfval != otherval {
                    return false;
                }
            }
        }
        // if neither both the type and cards are equal, they're equal
        true
    }
}
impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    /// compare two hands and decide which scores better
    fn cmp(&self, other: &Self) -> Ordering {
        // first compare their types
        if self.get_type().cmp(&other.get_type()) != Ordering::Equal {
            return self.get_type().cmp(&other.get_type());
        } else {
            // if they're the same type, compare each card
            for (selfcard, othercard) in iter::zip(self.cards.iter(), other.cards.iter()) {
                match get_card_strength(selfcard).cmp(&get_card_strength(othercard)) {
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Less => return Ordering::Less,
                    Ordering::Equal => continue,
                }
            }
        }
        // if type and cards are the same, they're equal
        Ordering::Equal
    }
}
