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
        'T' => 9,
        '9' => 8,
        '8' => 7,
        '7' => 6,
        '6' => 5,
        '5' => 4,
        '4' => 3,
        '3' => 2,
        '2' => 1,
        'J' => 0,
        _ => panic!("called get_card_strength on invalid card character"),
    }
}
#[derive(PartialEq, Eq, Debug)]
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

#[derive(Debug)]
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
        let counts = self.get_card_counts();
        // see if we can add the number of jokers to the number of another to get 5 of the same
        counts[1..]
            .iter()
            .max()
            .expect("invalid hand: too few cards")
            + counts[0]
            == 5
    }
    fn is_four_of_a_kind(&self) -> bool {
        let counts = self.get_card_counts();
        // see if we can add the number of jokers to the number of another to get 4 of the same
        counts[1..]
            .iter()
            .max()
            .expect("invalid hand: too few cards")
            + counts[0]
            == 4
    }
    fn is_full_house(&self) -> bool {
        if self.is_five_of_a_kind() || self.is_four_of_a_kind() {
            return false;
        }
        let counts = self.get_card_counts();
        let jokers = counts[0];
        // the number of jokers must be 2 or fewer, otherwise they'd score higher
        match jokers {
            0 => counts[1..].contains(&3) && counts[1..].contains(&2),
            1 => counts[1..].iter().filter(|n| **n == 2).count() == 2,
            2 => (counts[1..].contains(&1) && counts[1..].contains(&2)) || counts[1..].contains(&3),
            _ => unreachable!("if jokers is higher than 2, the check above will return"),
        }
    }
    fn is_three_of_a_kind(&self) -> bool {
        let counts = self.get_card_counts();
        !self.is_full_house()
            && counts[1..]
                .iter()
                .max()
                .expect("invalid hand, too few cards")
                + counts[0]
                == 3
    }
    fn is_two_pair(&self) -> bool {
        let counts = self.get_card_counts();
        if self.is_five_of_a_kind()
            || self.is_four_of_a_kind()
            || self.is_full_house()
            || self.is_three_of_a_kind()
        {
            return false;
        }
        let jokers = counts[0];
        // jokers must be 1 or zero at this point
        match jokers {
            0 => counts[1..].iter().filter(|c| **c == 2).count() == 2,
            1 => {
                counts[1..]
                    .iter()
                    .max()
                    .expect("invalid hand: too few cards")
                    == &2
            }
            _ => unreachable!("if jokers is greater than 1, the check above will return"),
        }
    }
    fn is_one_pair(&self) -> bool {
        let counts = self.get_card_counts();
        match counts.iter().max().expect("invalid hand: too few cards") {
            1 => counts[0] == 1,
            2 => counts[0] == 0 && counts[1..].iter().filter(|n| **n == 2).count() == 1,
            _ => false,
        }
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
