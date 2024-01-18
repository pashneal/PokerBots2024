use crate::constants::*;
use crate::distribution::Categorical;
use crate::eval::rank::HandRanker;
use crate::game_logic::action::*;
use crate::game_logic::state::{ActivePlayer, State};
use crate::game_logic::visibility::*;
use rand::prelude::*;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelativeSize {
    DeciPercent(u32),
    Amount(u32),
}
use RelativeSize::*;

impl RelativeSize {
    pub fn to_percent(&self, pot: u32) -> u32 {
        let size = match self {
            DeciPercent(p) => *p,
            Amount(a) => ((*a as f32 / pot as f32) * 1000.0).round() as u32,
        };
        size
    }

    pub fn to_amount(&self, pot: u32) -> u32 {
        let size = match self {
            DeciPercent(p) => (pot as f32 * (*p as f32 / 1000.0)).round() as u32,
            Amount(a) => *a,
        };
        size
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Parsable for Suit {
    fn to_string(&self) -> Option<String> {
        match self {
            Suit::Hearts => Some("h".to_string()),
            Suit::Diamonds => Some("d".to_string()),
            Suit::Clubs => Some("c".to_string()),
            Suit::Spades => Some("s".to_string()),
        }
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Suit {
    fn new(s: String) -> Self {
        match s.as_str() {
            "h" => Suit::Hearts,
            "d" => Suit::Diamonds,
            "c" => Suit::Clubs,
            "s" => Suit::Spades,
            _ => panic!("Invalid suit string"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Parsable for Value {
    fn to_string(&self) -> Option<String> {
        let result = match self {
            Value::Two => "2".to_string(),
            Value::Three => "3".to_string(),
            Value::Four => "4".to_string(),
            Value::Five => "5".to_string(),
            Value::Six => "6".to_string(),
            Value::Seven => "7".to_string(),
            Value::Eight => "8".to_string(),
            Value::Nine => "9".to_string(),
            Value::Ten => "T".to_string(),
            Value::Jack => "J".to_string(),
            Value::Queen => "Q".to_string(),
            Value::King => "K".to_string(),
            Value::Ace => "A".to_string(),
        };
        Some(result)
    }
    fn to_usize(&self) -> Option<usize> {
        let value = match self {
            Value::Ace => 0,
            Value::King => 1,
            Value::Queen => 2,
            Value::Jack => 3,
            Value::Ten => 4,
            Value::Nine => 5,
            Value::Eight => 6,
            Value::Seven => 7,
            Value::Six => 8,
            Value::Five => 9,
            Value::Four => 10,
            Value::Three => 11,
            Value::Two => 12,
        };
        Some(value)
    }
}

impl Value {
    fn new(s: String) -> Self {
        match s.as_str() {
            "2" => Value::Two,
            "3" => Value::Three,
            "4" => Value::Four,
            "5" => Value::Five,
            "6" => Value::Six,
            "7" => Value::Seven,
            "8" => Value::Eight,
            "9" => Value::Nine,
            "T" => Value::Ten,
            "J" => Value::Jack,
            "Q" => Value::Queen,
            "K" => Value::King,
            "A" => Value::Ace,
            _ => panic!("Invalid value string"),
        }
    }
}
impl From<usize> for Value {
    fn from(value: usize) -> Self {
        match value {
            0 => Value::Ace,
            1 => Value::King,
            2 => Value::Queen,
            3 => Value::Jack,
            4 => Value::Ten,
            5 => Value::Nine,
            6 => Value::Eight,
            7 => Value::Seven,
            8 => Value::Six,
            9 => Value::Five,
            10 => Value::Four,
            11 => Value::Three,
            12 => Value::Two,
            _ => panic!("Invalid suit index"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

impl Parsable for Card {
    fn to_string(&self) -> Option<String> {
        let string = format!(
            "{}{}",
            self.value.to_string().unwrap(),
            self.suit.to_string().unwrap()
        );
        Some(string)
    }
    fn to_usize(&self) -> Option<usize> {
        let suit = match self.suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 1,
            Suit::Clubs => 2,
            Suit::Spades => 3,
        };
        let value = match self.value {
            Value::Ace => 0,
            Value::King => 1,
            Value::Queen => 2,
            Value::Jack => 3,
            Value::Ten => 4,
            Value::Nine => 5,
            Value::Eight => 6,
            Value::Seven => 7,
            Value::Six => 8,
            Value::Five => 9,
            Value::Four => 10,
            Value::Three => 11,
            Value::Two => 12,
        };
        let index = suit + value * 4;
        Some(index)
    }
}
pub type CardIndex = usize;
impl Card {
    pub fn from_index(index: CardIndex) -> Self {
        let suit = match index % 4 {
            0 => Suit::Hearts,
            1 => Suit::Diamonds,
            2 => Suit::Clubs,
            3 => Suit::Spades,
            _ => panic!("Invalid value index"),
        };
        let value = match index / 4 {
            0 => Value::Ace,
            1 => Value::King,
            2 => Value::Queen,
            3 => Value::Jack,
            4 => Value::Ten,
            5 => Value::Nine,
            6 => Value::Eight,
            7 => Value::Seven,
            8 => Value::Six,
            9 => Value::Five,
            10 => Value::Four,
            11 => Value::Three,
            12 => Value::Two,
            _ => panic!("Invalid suit index"),
        };
        Card { value, suit }
    }
    pub fn new(s: &str) -> Self {
        let mut chars = s.chars();
        let value = Value::new(chars.next().unwrap().to_string());
        let suit = Suit::new(chars.next().unwrap().to_string());
        Card { value, suit }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hand {
    pub hand_size: usize,
    pub cards: Vec<Card>,
}

impl Parsable for Hand {
    fn to_string(&self) -> Option<String> {
        // Sort the cards so that the order is always the same
        let mut s = String::new();
        for card in self.cards.clone() {
            s.push_str(&card.to_string().unwrap());
        }
        Some(s)
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Hand {
    fn new() -> Self {
        Hand {
            hand_size: 2,
            cards: Vec::new(),
        }
    }
    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
        self.cards.sort_by(|a, b| {
            a.value
                .to_string()
                .unwrap()
                .cmp(&b.value.to_string().unwrap())
        });
    }

    fn expand(&mut self) {
        self.hand_size = match self.hand_size {
            2 => 3,
            _ => panic!("Cannot expand this hand size anymore!"),
        }
    }

    fn needs_hole_cards(&self) -> bool {
        self.hand_size > self.cards.len()
    }

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn as_u8(&self) -> Box<[u8]> {
        let mut result = Vec::new();
        for card in self.cards.clone() {
            result.push(card.to_usize().unwrap() as u8);
        }
        result.clone().into_boxed_slice()
    }

    fn cards(&self) -> Vec<Card> {
        self.cards.clone()
    }
}

fn card_features(cards: &Vec<Card>) -> Vec<Feature> {
    // See if the hand is suited (both cards are the same suit)
    let suited = cards[0].suit == cards[1].suit;
    // Sort the cards ranks by value
    let mut value = cards
        .clone()
        .iter()
        .map(|card| card.value.to_usize().unwrap())
        .collect::<Vec<usize>>();
    value.sort();
    let features = vec![
        Feature::Order(Round::PreFlop),
        Feature::Ranks(value[0], value[1]), 
        Feature::Suited(suited)
    ];
    features
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Winner {
    Player(usize),
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuctionPokerAction {
    Fold,
    Call,
    Check,
    Raise(RelativeSize),
    Bid(RelativeSize),                   //  representing an auction size from one of the players

    DealHole(CardIndex, usize), // Card dealt, player index
    DealCommunity(CardIndex),   // Deals a community card to the board

    ///////////////
    // These are not really "actions" they don't change the game state
    // but they are markers that make it easier to process and extract
    // features and insights from the game
    ///////////////
    BettingRoundStart,
    BettingRoundEnd,
    AuctionStart,
    PlayerActionEnd(usize),
    Auction(Winner),
}

impl Parsable for AuctionPokerAction {
    fn to_string(&self) -> Option<String> {
        None
    }

    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Into<ActionIndex> for AuctionPokerAction {
    fn into(self) -> ActionIndex {
        match self {
            AuctionPokerAction::Fold => 0,
            AuctionPokerAction::Call => 1,
            AuctionPokerAction::Check => 2,

            // We do a much smaller number of bet sizes
            AuctionPokerAction::Raise(DeciPercent(size)) => {
                match size {
                    // SMALL ABSTRACTION SPACE SO WE CAN TEST
                    // WHETHER THE ABSTRACTION IS WORKING
                    0..=300 => 3,
                    ..=500 => 4,
                    ..=600 => 5,
                    ..=750 => 6,
                    ..=1000 => 7,
                    ..=1250 => 8,
                    ..=1500 => 9,
                    ..=1750 => 10,
                    ..=2000 => 11,
                    ..=3000 => 12,
                    ..=4000 => 13,
                    ..=5000 => 14,
                    ..=10000 => 15,
                    ..=20000 => 16,
                    ..=30000 => 17,
                    ..=40000 => 18,
                    ..=50000 => 19,
                    ..=75000 => 20,
                    ..=1000000 => 21,
                    // LARGE ABSTRACTIONS
                    //// Get really granular for the first several sizes of the pot
                    //0..=50 => 3,
                    //..=100 => 4,
                    //..=150 => 5,
                    //..=200 => 6,
                    //..=250 => 7,
                    //..=300 => 8,
                    //..=350 => 9,
                    //..=400 => 10,
                    //..=450 => 11,
                    //..=500 => 12,
                    //..=550 => 13,
                    //..=600 => 14,
                    //..=650 => 15,
                    //..=700 => 16,
                    //..=750 => 17,
                    //..=800 => 18,
                    //..=850 => 19,
                    //..=900 => 20,
                    //..=950 => 21,
                    //..=1000 => 22,
                    //..=1050 => 23,
                    //..=1100 => 24,
                    //// Get less granular for the rest of the pot sizes
                    //..=1200 => 25,
                    //..=1500 => 26,
                    //..=2000 => 27,
                    //..=2500 => 28,
                    //..=3000 => 29,
                    //..=3500 => 30,
                    //..=4000 => 31,
                    //// Get wiggy with it
                    //..=5000 => 32,
                    //..=6000 => 33,
                    //..=7000 => 34,
                    //..=9000 => 35,
                    //..=10000 => 36,
                    //// Okay, now we're just being silly
                    //..=15000 => 37,
                    //..=25000 => 38,
                    //..=50000 => 39,
                    //..=100000 => 40,
                    //// This is just ridiculous, but necessary to capture all-ins
                    //// (all ins on preflop are ~13300% of pot)
                    //..=1000000 => 41,
                    _ => panic!("Well this is awkward... the bet size is too large!"),
                }
            }

            AuctionPokerAction::Raise(Amount(x)) => panic!(
                "Cannot convert raise size (amount) to action index! Convert to percent first!
                {:#?}",
                self.clone()
            ),

            AuctionPokerAction::Bid(Amount(x)) => { match x {
                0 => 22,
                ..=10 => 23,
                ..=20 => 24,
                ..=30 => 25,
                ..=40 => 26,
                ..=50 => 27,
                ..=60 => 28,
                ..=70 => 29,
                ..=80 => 30,
                ..=90 => 31,
                ..=110 => 32,
                ..=133 => 33,
                ..=150 => 34,
                ..=186 => 35,
                ..=195 => 36,
                ..=230 => 37,
                ..=356 => 38,
                ..=400 => 39,
                //0 => 42,
                //1..=10 => 43,
                //11..=20 => 44,
                //21..=30 => 45,
                //31..=40 => 46,
                //41..=50 => 47,
                //51..=60 => 48,
                //61..=70 => 49,
                //71..=80 => 50,
                //81..=90 => 51,
                //91..=100 => 52,
                //101..=110 => 53,
                //111..=120 => 54,
                //121..=130 => 55,
                //131..=140 => 56,
                //141..=150 => 57,
                //151..=160 => 58,
                //161..=170 => 59,
                //171..=180 => 60,
                //181..=190 => 61,
                //191..=200 => 62,
                //201..=210 => 63,
                //211..=220 => 64,
                //221..=230 => 65,
                //231..=240 => 66,
                //241..=250 => 67,
                //251..=260 => 68,
                //261..=270 => 69,
                //271..=280 => 70,
                //281..=290 => 71,
                //291..=300 => 72,
                //301..=310 => 73,
                //311..=320 => 74,
                //321..=330 => 75,
                //331..=340 => 76,
                //341..=350 => 77,
                //351..=360 => 78,
                //361..=370 => 79,
                //371..=380 => 80,
                //381..=390 => 81,
                //391..=400 => 82,
                _ => panic!("Well this is awkward... the bid size is too large!"),
            }}

            AuctionPokerAction::Bid(DeciPercent(_)) => panic!(
                "Cannot convert bid size (percent) to action index! Convert to amount first!"
            ),

            ///////////////////////
            // These should not matter because they are just markers
            // for the game state or performed by the Chance node
            ///////////////////////
            AuctionPokerAction::Auction(_) => 100,
            AuctionPokerAction::DealHole(_, _) => 100,
            AuctionPokerAction::DealCommunity(_) => 100,
            AuctionPokerAction::BettingRoundStart => 100,
            AuctionPokerAction::BettingRoundEnd => 100,
            AuctionPokerAction::AuctionStart => 100,
            AuctionPokerAction::PlayerActionEnd(_) => 100,
        }
    }
}

impl From<ActionIndex> for AuctionPokerAction {
    fn from(index: ActionIndex) -> Self {
        match index {
            0 => AuctionPokerAction::Fold,
            1 => AuctionPokerAction::Call,
            2 => AuctionPokerAction::Check,
            3 => AuctionPokerAction::Raise(DeciPercent(30)),
            4 => AuctionPokerAction::Raise(DeciPercent(80)),
            5 => AuctionPokerAction::Raise(DeciPercent(130)),
            6 => AuctionPokerAction::Raise(DeciPercent(180)),
            7 => AuctionPokerAction::Raise(DeciPercent(230)),
            8 => AuctionPokerAction::Raise(DeciPercent(280)),
            9 => AuctionPokerAction::Raise(DeciPercent(330)),
            10 => AuctionPokerAction::Raise(DeciPercent(380)),
            11 => AuctionPokerAction::Raise(DeciPercent(430)),
            12 => AuctionPokerAction::Raise(DeciPercent(480)),
            13 => AuctionPokerAction::Raise(DeciPercent(530)),
            14 => AuctionPokerAction::Raise(DeciPercent(580)),
            15 => AuctionPokerAction::Raise(DeciPercent(630)),
            16 => AuctionPokerAction::Raise(DeciPercent(680)),
            17 => AuctionPokerAction::Raise(DeciPercent(730)),
            18 => AuctionPokerAction::Raise(DeciPercent(780)),
            19 => AuctionPokerAction::Raise(DeciPercent(830)),
            20 => AuctionPokerAction::Raise(DeciPercent(880)),
            21 => AuctionPokerAction::Raise(DeciPercent(930)),
            22 => AuctionPokerAction::Raise(DeciPercent(980)),
            23 => AuctionPokerAction::Raise(DeciPercent(1030)),
            24 => AuctionPokerAction::Raise(DeciPercent(1080)),
            // Get less granular for the rest of the pot sizes
            25 => AuctionPokerAction::Raise(DeciPercent(1160)),
            26 => AuctionPokerAction::Raise(DeciPercent(1360)),
            27 => AuctionPokerAction::Raise(DeciPercent(1750)),
            28 => AuctionPokerAction::Raise(DeciPercent(2250)),
            29 => AuctionPokerAction::Raise(DeciPercent(2750)),
            30 => AuctionPokerAction::Raise(DeciPercent(3250)),
            31 => AuctionPokerAction::Raise(DeciPercent(3750)),
            // Get wiggy with it
            32 => AuctionPokerAction::Raise(DeciPercent(4500)),
            33 => AuctionPokerAction::Raise(DeciPercent(5500)),
            34 => AuctionPokerAction::Raise(DeciPercent(6500)),
            35 => AuctionPokerAction::Raise(DeciPercent(8000)),
            36 => AuctionPokerAction::Raise(DeciPercent(9500)),
            // Okay, now we're just being silly
            37 => AuctionPokerAction::Raise(DeciPercent(12500)),
            38 => AuctionPokerAction::Raise(DeciPercent(20000)),
            39 => AuctionPokerAction::Raise(DeciPercent(37500)),
            40 => AuctionPokerAction::Raise(DeciPercent(75000)),
            // This is just ridiculous, but necessary to capture all-ins
            // (all ins on preflop are ~13300% of pot0)
            41 => AuctionPokerAction::Raise(DeciPercent(500000)),
            42 => AuctionPokerAction::Bid(Amount(0)),
            43 => AuctionPokerAction::Bid(Amount(5)),
            44 => AuctionPokerAction::Bid(Amount(15)),
            45 => AuctionPokerAction::Bid(Amount(25)),
            46 => AuctionPokerAction::Bid(Amount(36)),
            47 => AuctionPokerAction::Bid(Amount(45)),
            48 => AuctionPokerAction::Bid(Amount(55)),
            49 => AuctionPokerAction::Bid(Amount(65)),
            50 => AuctionPokerAction::Bid(Amount(75)),
            51 => AuctionPokerAction::Bid(Amount(85)),
            52 => AuctionPokerAction::Bid(Amount(95)),
            53 => AuctionPokerAction::Bid(Amount(105)),
            54 => AuctionPokerAction::Bid(Amount(115)),
            55 => AuctionPokerAction::Bid(Amount(125)),
            56 => AuctionPokerAction::Bid(Amount(135)),
            57 => AuctionPokerAction::Bid(Amount(145)),
            58 => AuctionPokerAction::Bid(Amount(155)),
            59 => AuctionPokerAction::Bid(Amount(165)),
            60 => AuctionPokerAction::Bid(Amount(175)),
            61 => AuctionPokerAction::Bid(Amount(185)),
            62 => AuctionPokerAction::Bid(Amount(195)),
            63 => AuctionPokerAction::Bid(Amount(205)),
            64 => AuctionPokerAction::Bid(Amount(215)),
            65 => AuctionPokerAction::Bid(Amount(225)),
            66 => AuctionPokerAction::Bid(Amount(235)),
            67 => AuctionPokerAction::Bid(Amount(245)),
            68 => AuctionPokerAction::Bid(Amount(255)),
            69 => AuctionPokerAction::Bid(Amount(265)),
            70 => AuctionPokerAction::Bid(Amount(275)),
            71 => AuctionPokerAction::Bid(Amount(285)),
            72 => AuctionPokerAction::Bid(Amount(295)),
            73 => AuctionPokerAction::Bid(Amount(305)),
            74 => AuctionPokerAction::Bid(Amount(315)),
            75 => AuctionPokerAction::Bid(Amount(325)),
            76 => AuctionPokerAction::Bid(Amount(335)),
            77 => AuctionPokerAction::Bid(Amount(345)),
            78 => AuctionPokerAction::Bid(Amount(355)),
            79 => AuctionPokerAction::Bid(Amount(365)),
            80 => AuctionPokerAction::Bid(Amount(375)),
            81 => AuctionPokerAction::Bid(Amount(385)),
            82 => AuctionPokerAction::Bid(Amount(395)),
            83 => AuctionPokerAction::Bid(Amount(405)),
            _ => panic!("No"),
        }
    }
}

impl Filterable for AuctionPokerAction {}
impl Action for AuctionPokerAction {
    fn max_index() -> ActionIndex {
        83
    }
    fn index(&self) -> ActionIndex {
        self.clone().into()
    }
}

#[derive(Debug, Clone)]
pub struct AuctionPokerState {
    card_bits: u64,
    bids: [Option<u32>; 2],
    player_hands: [Hand; 2],
    community_cards: Vec<Card>,
    pot: u32,
    pips: [u32; 2], // Amount of money each player has put into the pot per betting round
    stacks: [u32; 2],
    raise: Option<u32>, // Cost of the last raise
    active_player: ActivePlayer<AuctionPokerAction>,
    winner: Option<Winner>, // Winner of a bid
    cached_ev: [[Option<f32>; 2]; 5],
    aggression : usize,
}

impl AuctionPokerState {
    fn current_betting_round(&self) -> Round {
        match self.community_cards.len() {
            0 => Round::PreFlop,
            3 => Round::Flop,
            4 => Round::Turn,
            5 => Round::River,
            _ => panic!("Not a legal betting round!"),
        }
    }
    fn pre_bid_observations(&self) -> Vec<Observation<AuctionPokerAction>> {
        let community_cards: Vec<u8> = self
            .community_cards
            .iter()
            .map(|x| x.to_usize().unwrap() as u8)
            .collect();
        let ranker = HandRanker::new();
        let iterations = EV_ITERATIONS;

        // Calculate consequences if player 0 lost or
        // won the upcoming bid on the flop
        let hand = self.player_hands[0].cards();
        let hand: Vec<u8> = hand.iter().map(|x| x.to_usize().unwrap() as u8).collect();
        let ev_win0 = ranker.rollout_bid_win(&hand, &community_cards, iterations);
        let ev_loss0 = ranker.rollout_bid_loss(&hand, &community_cards, iterations);

        // And the same for player 1
        let hand = self.player_hands[1].cards();
        let hand: Vec<u8> = hand.iter().map(|x| x.to_usize().unwrap() as u8).collect();
        let ev_win1 = ranker.rollout_bid_win(&hand, &community_cards, iterations);
        let ev_loss1 = ranker.rollout_bid_loss(&hand, &community_cards, iterations);

        // ALWAYS truncate, it would be very bad
        // to think that we have the nuts when we don't
        let ev_win0 = (ev_win0 * 30.0) as u16;
        let ev_win1 = (ev_win1 * 30.0) as u16;
        let ev_loss0 = (ev_loss0 * 30.0) as u16;
        let ev_loss1 = (ev_loss1 * 30.0) as u16;

        let pot = self.pot as f32 / MAX_POT as f32;
        let pot = (pot * 20.0) as u8;

        let p0_features = vec![
            Feature::Order(Round::Auction),
            Feature::EV(ev_loss0),
            Feature::EV(ev_win0),
            Feature::Pot(pot),
        ];
        let p1_features = vec![
            Feature::Order(Round::Auction),
            Feature::EV(ev_loss1),
            Feature::EV(ev_win1),
            Feature::Pot(pot),
        ];

        vec![
            Observation::Shared(Information::Features(p0_features), vec![0]),
            Observation::Shared(Information::Features(p1_features), vec![1]),
        ]
    }

    fn round_features(&mut self, round: &Round, player_num: usize) -> Vec<Feature> {
        let round = round.clone();
        if matches!(round, Round::PreFlop) {
            return vec![];
        };

        let time = Instant::now();
        let ev = self.get_player_ev(&round, player_num);

        let ev = (ev * 50.0) as u16;
        let winner = match self.winner {
            Some(Winner::Player(0)) => BidResult::Player(0),
            Some(Winner::Player(_)) => BidResult::Player(1),
            Some(Winner::Tie) => BidResult::Tie,
            None => panic!("There should be a winner by now!"),
        };
        let features = vec![
            Feature::Order(round),
            Feature::EV(ev),
            Feature::Aggression(self.aggression),
            Feature::Auction(winner),
        ];
        features
    }

    // Calculate the EV of a player's hand at a given round and cache + return it
    fn get_player_ev(&mut self, round: &Round, player_num: usize) -> f32 {
        // If we've already calculated the ev, return it
        let round_index: usize = round.clone().into();
        if let Some(ev) = self.cached_ev[round_index][player_num] {
            return ev;
        }

        let ranker = HandRanker::new();
        let iterations = EV_ITERATIONS;

        let hand = self.player_hands[player_num].cards();

        let hand: Vec<u8> = hand.iter().map(|x| x.to_usize().unwrap() as u8).collect();
        let community_cards: Vec<u8> = self
            .community_cards
            .iter()
            .map(|x| x.to_usize().unwrap() as u8)
            .collect();

        const REDUCE: u32 = 2;
        // Note: The reason we divide by REDUCE on the river is
        // because accuracy can be sacrificed for speed
        // (fewer card possibilities to sample from)
        let ev = match self.winner {
            Some(Winner::Player(winner_num)) if winner_num == player_num => {
                let ev_won = match round {
                    Round::Flop => ranker.rollout_flop_won(&hand, &community_cards, iterations),
                    Round::Turn => ranker.rollout_turn_won(&hand, &community_cards, iterations),
                    Round::River => {
                        ranker.rollout_river_won(&hand, &community_cards, iterations / REDUCE)
                    }
                    _ => panic!("Cannot evaluate ev on this round"),
                };
                ev_won
            }
            Some(Winner::Player(_)) => {
                let ev_lost = match round {
                    Round::Flop => ranker.rollout_flop_lost(&hand, &community_cards, iterations),
                    Round::Turn => ranker.rollout_turn_lost(&hand, &community_cards, iterations),
                    Round::River => {
                        ranker.rollout_river_lost(&hand, &community_cards, iterations / REDUCE)
                    }
                    _ => panic!("Cannot evaluate ev on this round"),
                };
                ev_lost
            }
            Some(Winner::Tie) => {
                let ev_tie = match round {
                    Round::Flop => ranker.rollout_flop_tie(&hand, &community_cards, iterations),
                    Round::Turn => ranker.rollout_turn_tie(&hand, &community_cards, iterations),
                    Round::River => {
                        ranker.rollout_river_tie(&hand, &community_cards, iterations / REDUCE)
                    }
                    _ => panic!("Cannot evaluate ev on this round"),
                };
                ev_tie
            }
            None => panic!("Winner was not set after auction"),
        };

        let ev = ev as f32;
        self.cached_ev[round_index][player_num] = Some(ev);

        ev
    }

    fn needs_hole_cards(&self) -> bool {
        self.player_hands[0].needs_hole_cards() || self.player_hands[1].needs_hole_cards()
    }

    fn hole_card_dealer(&self) -> ActivePlayer<AuctionPokerAction> {
        let player_num = match self.player_hands[0].needs_hole_cards() {
            true => 0,
            false => 1,
        };

        let mut cards = Vec::new();
        for i in 0..52 {
            if self.card_bits & (1 << i) == 0 {
                cards.push(AuctionPokerAction::DealHole(i, player_num));
            }
        }

        ActivePlayer::Chance(Categorical::uniform(cards))
    }

    fn next_dealer(&self) -> ActivePlayer<AuctionPokerAction> {
        match self.community_cards.len() {
            0 => self.deal(),
            3 => self.deal(),
            4 => self.deal(),
            5 => self.showdown(),
            _ => panic!("Cannot deal to community when there are this many community cards."),
        }
    }

    fn initial_node() -> ActivePlayer<AuctionPokerAction> {
        let mut cards = Vec::new();
        for i in 0..52 {
            cards.push(AuctionPokerAction::DealHole(i, 0));
        }
        ActivePlayer::Chance(Categorical::uniform(cards))
    }

    fn deal(&self) -> ActivePlayer<AuctionPokerAction> {
        let mut cards = Vec::new();
        for i in 0..52 {
            if self.card_bits & (1 << i) == 0 {
                cards.push(AuctionPokerAction::DealCommunity(i));
            }
        }

        ActivePlayer::Chance(Categorical::uniform(cards))
    }

    fn betting_round_start(&self) -> ActivePlayer<AuctionPokerAction> {
        let start = AuctionPokerAction::BettingRoundStart;
        ActivePlayer::Marker(start)
    }
    fn betting_round_end(&self) -> ActivePlayer<AuctionPokerAction> {
        let end = AuctionPokerAction::BettingRoundEnd;
        ActivePlayer::Marker(end)
    }
    fn auction_start(&self) -> ActivePlayer<AuctionPokerAction> {
        let start = AuctionPokerAction::AuctionStart;
        ActivePlayer::Marker(start)
    }
    fn action_end(&self, player_num: usize) -> ActivePlayer<AuctionPokerAction> {
        let start = AuctionPokerAction::PlayerActionEnd(player_num);
        ActivePlayer::Marker(start)
    }

    fn betting_round(&self, player_num: usize) -> ActivePlayer<AuctionPokerAction> {
        // Amount needed to bet/raise instead of call
        // this represents the total amount of money needed
        let min_raise = match self.raise {
            Some(raise) => raise + self.pips[player_num ^ 1],
            None => BIG_BLIND + self.pips[player_num ^ 1],
        };

        let mut actions = Vec::new();

        // See variant rules: cannot raise more than either player's stack + pip
        let max_raise = (self.stacks[player_num] + self.pips[player_num]).min(self.stacks[player_num ^ 1] + self.pips[player_num ^ 1]);

        for i in min_raise..=max_raise {
            let raise_percent = Amount(i).to_percent(self.pot);
            actions.push(AuctionPokerAction::Raise(DeciPercent(raise_percent)));
        }

        // See poker rules:
        // even if the maximum raise is lower than the minimum raise,
        // the player can still go all in
        let current_stack = self.stacks[player_num];
        if current_stack > 0
            && actions.len() == 0
            && self.stacks[player_num] <= self.stacks[player_num ^ 1]
        {
            let raise_percent = Amount(current_stack + self.pips[player_num]).to_percent(self.pot);
            actions.push(AuctionPokerAction::Raise(DeciPercent(raise_percent)));
        }

        if self.pips[player_num] == self.pips[player_num ^ 1] {
            // Nobody raised, so we can check
            actions.push(AuctionPokerAction::Check);
        } else {
            // We can always call if the pips are unequal
            // (by virtue of never being able to raise more than the smaller stack)
            actions.push(AuctionPokerAction::Call);
            actions.push(AuctionPokerAction::Fold);
        }

        if self.aggression == AGGRESSION_LIMIT {
            actions  = actions.into_iter().filter(|action| !matches!(action ,AuctionPokerAction::Raise(_))).collect();
        }
        ActivePlayer::Player(player_num as u32, actions)
    }

    fn auction_continue(&self) -> ActivePlayer<AuctionPokerAction> {
        let player0_bids = (0..=self.stacks[0])
            .map(|x| {
                AuctionPokerAction::Bid(Amount(x))
            })
            .collect::<Vec<_>>();
        let player1_bids = (0..=self.stacks[1])
            .map(|x| {
                AuctionPokerAction::Bid(Amount(x))
            })
            .collect::<Vec<_>>();

        match self.bids {
            [None, None] => ActivePlayer::Player(1, player0_bids),
            [None, Some(_)] => ActivePlayer::Player(0, player1_bids),
            [Some(bid0), Some(bid1)] => {
                let winner: Winner = if bid0 > bid1 {
                    Winner::Player(0)
                } else if bid1 > bid0 {
                    Winner::Player(1)
                } else {
                    Winner::Tie
                };
                ActivePlayer::Marker(AuctionPokerAction::Auction(winner))
            }
            _ => panic!("Invalid bids states"),
        }
    }

    /// One of the two players folded
    fn folded(&self, player_num: usize) -> ActivePlayer<AuctionPokerAction> {
        let contribution = STACK_SIZE - self.stacks[player_num];
        let delta = contribution as f32;
        match player_num {
            0 => ActivePlayer::Terminal(vec![-delta, delta]),
            1 => ActivePlayer::Terminal(vec![delta, -delta]),
            _ => panic!("Invalid player number"),
        }
    }

    /// The game is over, determine the winner
    fn showdown(&self) -> ActivePlayer<AuctionPokerAction> {
        let mut player0 = self.player_hands[0].clone();
        let mut player1 = self.player_hands[1].clone();

        for card in &self.community_cards {
            player0.add_card(card.clone());
            player1.add_card(card.clone());
        }

        let player0_hand_len = player0.len();
        let player1_hand_len = player1.len();

        let hand_ranker = HandRanker::new();

        let player1_rank = match player1_hand_len {
            8 => hand_ranker.rank8(&player1.as_u8()),
            7 => hand_ranker.rank7(&player1.as_u8()),
            _ => panic!("Invalid hand + community length"),
        };

        let player0_rank = match player0_hand_len {
            8 => hand_ranker.rank8(&player0.as_u8()),
            7 => hand_ranker.rank7(&player0.as_u8()),
            _ => panic!("Invalid hand + community length"),
        };

        let contribution0 = STACK_SIZE - self.stacks[0];
        let contribution1 = STACK_SIZE - self.stacks[1];

        let contribution0 = contribution0 as f32;
        let contribution1 = contribution1 as f32;

        // See piazza: extra chip awarded to BB in an odd pot with a tie (BB always
        // second to play)
        let extra_chip = (self.pot % 2) as f32;
        let half_pot = (self.pot as f32 - extra_chip) / 2.0;

        let deltas = match player0_rank.cmp(&player1_rank) {
            Ordering::Greater => vec![contribution1, -contribution1],
            Ordering::Less => vec![-contribution0, contribution0],
            Ordering::Equal => vec![
                contribution0 - half_pot,
                contribution1 - half_pot + extra_chip,
            ],
        };

        ActivePlayer::Terminal(deltas)
    }

    fn new_pot_after(&self, action: &AuctionPokerAction) -> u32 {
        match action {
            AuctionPokerAction::Call => {
                let max_pip = self.pips[0].max(self.pips[1]);
                self.pot + (max_pip - self.pips[0]) + (max_pip - self.pips[1])
            }
            AuctionPokerAction::Fold => self.pot,
            AuctionPokerAction::Raise(size) => {
                let size = size.to_amount(self.pot);
                let player_num = self.active_player().player_num();
                let cost = size - self.pips[player_num];
                self.pot + cost
            }
            AuctionPokerAction::Auction(winner) => match winner {
                Winner::Player(player_num) => self.pot + self.bids[player_num ^ 1].unwrap(),
                Winner::Tie => self.pot + 2*self.bids[0].unwrap(),
            },
            _ => todo!(),
        }
    }
}

impl State<AuctionPokerAction> for AuctionPokerState {
    fn new() -> Self {
        AuctionPokerState {
            card_bits: 0,
            bids: [None, None],
            player_hands: [Hand::new(), Hand::new()],
            pot: LITTLE_BLIND + BIG_BLIND,
            community_cards: Vec::new(),
            stacks: [400 - LITTLE_BLIND, 400 - BIG_BLIND],
            pips: [1, 2],
            raise: Some(2),
            active_player: AuctionPokerState::initial_node(),
            winner: None,
            cached_ev: [[None, None]; 5],
            aggression : 0,
        }
    }

    fn get_observations_after(
        &mut self,
        action: &AuctionPokerAction,
    ) -> Vec<Observation<AuctionPokerAction>> {
        match action {
            AuctionPokerAction::Fold => {
                // Doesn't really matter what happens here, since the game is over
                vec![Observation::Public(Information::Discard)]
            }

            AuctionPokerAction::Call => {
                // Don't really care what happens here
                vec![Observation::Public(Information::Discard)]
            }
            AuctionPokerAction::Check => {
                // Don't really care what happens here
                vec![Observation::Public(Information::Discard)]
            }
            AuctionPokerAction::DealHole(_, player_num) => {
                // Share the private information only with the player who got the card
                let observation =
                    Observation::Shared(Information::Action(action.clone()), vec![*player_num]);
                vec![observation]
            }
            AuctionPokerAction::DealCommunity(_) => {
                vec![Observation::Public(Information::Action(action.clone()))]
            }
            AuctionPokerAction::Raise(_) => {
                // Don't really care what happens here
                vec![Observation::Public(Information::Discard)]
            }
            AuctionPokerAction::Bid(_) => {
                // Don't really care what happens here
                vec![Observation::Public(Information::Discard)]
            }
            AuctionPokerAction::Auction(_) => {
                vec![Observation::Private(Information::Action(action.clone()))]
            }

            //////////////////////////////
            // These update the feature set!
            //////////////////////////////
            AuctionPokerAction::BettingRoundStart | AuctionPokerAction::PlayerActionEnd(_) => {
                // TODO: slight optimization with only updating the specific player under
                // PlayerActionEnd

                let pot = self.pot;
                let pot = pot as f32 / MAX_POT as f32;
                let scaled_pot = (pot * 100.0) as u8;
                let stacks = [
                    self.stacks[0] as f32 / STACK_SIZE as f32,
                    self.stacks[1] as f32 / STACK_SIZE as f32,
                ];
                let scaled_stacks = [(stacks[0] * 30.0) as u8, (stacks[1] * 30.0) as u8];

                let pot_and_stacks = [
                    Feature::Pot(scaled_pot),
                    Feature::Stack(scaled_stacks[0]),
                    Feature::Stack(scaled_stacks[1]),
                ];

                let round = self.current_betting_round();
                let mut features0 = self.round_features(&round, 0);
                let mut features1 = self.round_features(&round, 1);

                features0.extend(pot_and_stacks.clone());
                features1.extend(pot_and_stacks);

                let features1 = Information::Features(features1);
                let features0 = Information::Features(features0);

                match round {
                    Round::PreFlop => {
                        // If it's a preflop we use the special case instead
                        // (only cards + pot)
                        let mut features0 = card_features(&self.player_hands[0].cards());
                        let mut features1 = card_features(&self.player_hands[1].cards());
                        features0.push(Feature::Aggression(self.aggression));
                        features1.push(Feature::Aggression(self.aggression));
                        features0.push(Feature::Pot(scaled_pot));
                        features1.push(Feature::Pot(scaled_pot));

                        let features0 = Information::Features(features0);
                        let features1 = Information::Features(features1);

                        vec![
                            Observation::Shared(features0, vec![0]),
                            Observation::Shared(features1, vec![1]),
                        ]
                    }
                    Round::Turn | Round::River | Round::Flop => {
                        vec![
                            Observation::Shared(features0, vec![0]),
                            Observation::Shared(features1, vec![1]),
                        ]
                    }
                    _ => panic!("Cannot start betting during this round!"),
                }
            }
            AuctionPokerAction::AuctionStart => self.pre_bid_observations(),

            AuctionPokerAction::BettingRoundEnd => {
                // Sanity check
                debug_assert!(self.pot + self.stacks[0] + self.stacks[1] == MAX_POT);
                // TODO: I don't think there's anything to be done here but may be wrong
                vec![Observation::Public(Information::Discard)]
            }
        }
    }

    fn active_player(&self) -> ActivePlayer<AuctionPokerAction> {
        return self.active_player.clone();
    }

    fn update(&mut self, action: AuctionPokerAction) {
        match action {
            AuctionPokerAction::Fold => {
                let player_num = self.active_player.player_num() as usize;
                self.active_player = self.folded(player_num);
            }
            AuctionPokerAction::Call => {
                let max_pip = self.pips.iter().max().unwrap();

                // Take the pip diff and subtract from both players
                self.stacks = [
                    self.stacks[0] - (max_pip - self.pips[0]),
                    self.stacks[1] - (max_pip - self.pips[1]),
                ];
                // add diffs to pot
                self.pot = self.new_pot_after(&AuctionPokerAction::Call);
                self.pips = [0, 0];
                self.raise = None;

                // Sanity check pot amounts
                debug_assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);

                self.active_player = self.betting_round_end();
            }
            AuctionPokerAction::Check => {
                let player_num = self.active_player.player_num() as usize;
                match self.current_betting_round() {
                    Round::PreFlop => match player_num {
                        0 => self.active_player = self.action_end(player_num),
                        1 => self.active_player = self.betting_round_end(),
                        _ => panic!("Invalid player number"),
                    }
                    Round::Flop | Round::Turn | Round::River => match player_num {
                        1 => self.active_player = self.action_end(player_num),
                        0 => self.active_player = self.betting_round_end(),
                        _ => panic!("Invalid player number"),
                    }
                    _ => panic!("Cannot check during this round!"),
                }
                debug_assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);
            }
            AuctionPokerAction::DealHole(card_index, player_num) => {
                let card = Card::from_index(card_index);
                self.player_hands[player_num].add_card(card);
                self.card_bits |= 1 << card_index;
                if self.needs_hole_cards() {
                    self.active_player = self.hole_card_dealer();
                } else {
                    // Start off the next round of betting!
                    self.active_player = self.betting_round_start();
                }
            }

            AuctionPokerAction::DealCommunity(card_index) => {
                self.community_cards.push(Card::from_index(card_index));
                self.card_bits |= 1 << card_index;
                let street = self.community_cards.len();
                let bidding_round_over = self.bids[1].is_some();
                self.active_player = match (street, bidding_round_over) {
                    (0..=2, _) => self.deal(),               // Not enough cards, deal again
                    (3, false) => self.auction_start(),      // Kick off bidding!
                    (3, true) => self.betting_round_start(), // Start betting rounds
                    (4, _) => self.betting_round_start(),
                    (5, _) => self.betting_round_start(),
                    _ => panic!("Unsure what to do after dealing in this situation"),
                }
            }

            AuctionPokerAction::Raise(size) => {
                let player_num = self.active_player().player_num();

                let amount = size.to_amount(self.pot);

                let cost = amount - self.pips[player_num];
                self.pot = self.new_pot_after(&AuctionPokerAction::Raise(Amount(amount)));
                self.pips[player_num] += cost;
                self.stacks[player_num] -= cost;

                // Opponent bet something - so this is a raise
                if self.pips[player_num ^ 1] > 0 {
                    self.raise = Some(cost);
                } else {
                    self.raise = None;
                }

                // Sanity check pot amounts
                debug_assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);

                self.aggression += 1;
                // End the action, but not the round
                self.active_player = self.action_end(player_num);
            }

            AuctionPokerAction::Bid(size) => {
                let bid = size.to_amount(self.pot);
                let player_num = self.active_player().player_num();
                self.bids[player_num] = Some(bid);
                self.active_player = self.auction_continue();
            }

            AuctionPokerAction::Auction(winner) => {
                match winner {
                    Winner::Player(player_num) => {
                        // Loser's bid goes in the pot and is taken from winner
                        self.stacks[player_num] -= self.bids[player_num ^ 1].unwrap();
                        // Winner gets another card!
                        self.player_hands[player_num].expand();
                    }
                    Winner::Tie => {
                        // Both players get another card!
                        self.player_hands[0].expand();
                        self.player_hands[1].expand();
                        // See variant: Both players lose their bids to the pot
                        self.stacks[0] -= self.bids[0].unwrap();
                        self.stacks[1] -= self.bids[0].unwrap();
                    }
                }
                self.winner = Some(winner.clone());
                self.pot = self.new_pot_after(&AuctionPokerAction::Auction(winner));

                // Sanity check pot amounts
                debug_assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);

                // Always needs to deal hole cards after an auction
                self.active_player = self.hole_card_dealer();
            }

            AuctionPokerAction::BettingRoundStart => {
                // Kick off the betting round with player 0 in PreFlop
                // and player 1 in Auction and onwards
                self.aggression = 0;
                match self.current_betting_round() {
                    Round::PreFlop => self.active_player = self.betting_round(0),
                    _ => self.active_player = self.betting_round(1),
                }
            }
            AuctionPokerAction::PlayerActionEnd(player_num) => {
                // Always transition the the other player,
                // if the betting round was over, then BettingRoundEnd will handle it
                // instead
                self.active_player = self.betting_round(player_num ^ 1);
            }

            AuctionPokerAction::BettingRoundEnd => {
                // We always need the dealer to do stuff
                // (deal community cards, deal hole cards, etc.)
                // when the betting rounds end
                self.raise = None;
                self.pips = [0, 0];
                self.active_player = self.next_dealer();
                assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);
            }

            AuctionPokerAction::AuctionStart => {
                // Note: This is action/marker is just a formality so that
                // observations and features at the start of the auction can be made
                // independently of the logic needed to update the game state
                self.active_player = self.auction_continue();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chance_transition() {
        let mut state = AuctionPokerState::new();
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 52);
                let action = chance.sample();
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 51);
                let action = chance.sample();
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 50);
                let action = chance.sample();
                match action {
                    AuctionPokerAction::DealHole(_, player_num) => {
                        assert_eq!(player_num, 1, "Expected player 1 to get the card next");
                    }
                    _ => panic!("Expected deal player action ."),
                }
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }
        // Glass box testing
        assert_eq!(state.card_bits.count_ones(), 3);
    }

    #[test]
    fn test_immediate_fold() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        for _ in 0..4 {
            let active_player = state.active_player();
            match active_player {
                ActivePlayer::Chance(chance) => {
                    let action = chance.sample();
                    state.update(action);
                }
                _ => panic!("Expected chance transition."),
            }
        }
        state.update(AuctionPokerAction::BettingRoundStart);
        // First player should be able to fold
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), true);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), true);
                // Cannot check on the first turn with only BB/LB in pot
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Fold);
        // Should be terminal state with player 1 winning LB
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Terminal(deltas) => {
                assert_eq!(deltas[0], -1.0); // Player 0 loses LB
                assert_eq!(deltas[1], 1.0); // Player 1 wins LB
            }
            x => panic!("Expected terminal state. Got {:?}", x),
        }
    }

    #[test]
    fn test_auction_tie() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(4)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        // Deal a bunch of community cards
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        println!("{:?}", state);

        state.update(AuctionPokerAction::Bid(Amount(20)));
        state.update(AuctionPokerAction::Bid(Amount(20)));

        assert_eq!(
            state
                .active_player()
                .actions()
                .contains(&AuctionPokerAction::Auction(Winner::Tie)),
            true
        );
    }

    #[test]
    fn test_auction() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));

        // Betting round checks
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(4)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::PlayerActionEnd(0)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Raise(Amount(50)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::PlayerActionEnd(1)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Call);
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundEnd));
        state.update(AuctionPokerAction::BettingRoundEnd);

        // Deal a bunch of community cards
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));

        // Auction round checks
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::AuctionStart));
        state.update(AuctionPokerAction::AuctionStart);

        println!("{:?}", state);

        match state.active_player() {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 1);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
                for i in 0..=350 {
                    // Should have 350 in stack after reraise and call
                    assert_eq!(
                        i <= state.stacks[0],
                        true,
                        "Expected i to match the stack size {}. Stack size : {}",
                        i,
                        state.stacks[0]
                    );
                    assert_eq!(
                        actions.contains(&AuctionPokerAction::Bid(Amount(i))),
                        true,
                        "Expected bid {} to be available",
                        i
                    );
                }
                assert_eq!(
                    {
                        actions.contains(&AuctionPokerAction::Bid(Amount(351)))
                    },
                    false,
                    "Expected bid 351 onwards to be unavailable"
                );
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Bid(Amount(20)));
        match state.active_player() {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
                for i in 0..=350 {
                    // glass box testing
                    assert_eq!(
                        i <= state.stacks[1],
                        true,
                        "Expected i to match the stack size {}. Stack size : {}",
                        i,
                        state.stacks[1]
                    );
                    assert_eq!(
                        actions.contains(&AuctionPokerAction::Bid(Amount(i))),
                        true,
                        "Expected bid {} to be available",
                        i
                    );
                }
                assert_eq!(
                    actions.contains(&AuctionPokerAction::Bid(Amount(351))),
                    false,
                    "Expected bid 351 to be unavailable"
                );
            }

            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Bid(Amount(300)));

        // Player 0 should have won the auction
        assert_eq!(
            state
                .active_player()
                .actions()
                .contains(&AuctionPokerAction::Auction(Winner::Player(0))),
            true
        );
    }


    #[test]
    fn test_flop_check_check() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(1)));
        state.update(AuctionPokerAction::Bid(Amount(0)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Auction(Winner::Player(1))));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));
        state.update(AuctionPokerAction::BettingRoundStart);

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 1);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), true);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::PlayerActionEnd(1));

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), true);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }
    }

    #[test]
    fn test_showdown() {
        // Should win the pot when dealt the nuts
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(
            Card::new("Ah").to_usize().unwrap(),
            0,
        ));
        state.update(AuctionPokerAction::DealHole(
            Card::new("Ac").to_usize().unwrap(),
            0,
        ));
        state.update(AuctionPokerAction::DealHole(
            Card::new("2c").to_usize().unwrap(),
            1,
        ));
        state.update(AuctionPokerAction::DealHole(
            Card::new("2h").to_usize().unwrap(),
            1,
        ));
        // Make sure that we go to the BettingRoundStart marker
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundStart));
        state.update(AuctionPokerAction::BettingRoundStart);

        // First betting round (pre-flop)
        state.update(AuctionPokerAction::Raise(Amount(9)));
        // Make sure that we go to the PlayerActionEnd marker
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::PlayerActionEnd(0))));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Call);

        // Make sure that we go to the BettingRoundEnd marker
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundEnd));

        // pot = 18

        // Flop dealt
        state.update(AuctionPokerAction::DealCommunity(
            Card::new("Ad").to_usize().unwrap(),
        ));
        state.update(AuctionPokerAction::DealCommunity(
            Card::new("As").to_usize().unwrap(),
        ));
        state.update(AuctionPokerAction::DealCommunity(
            Card::new("2d").to_usize().unwrap(),
        ));

        // Make sure that we go to the AuctionStart marker
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::AuctionStart));
        state.update(AuctionPokerAction::AuctionStart);

        // Auction starts
        state.update(AuctionPokerAction::Bid(Amount(25)));
        state.update(AuctionPokerAction::Bid(Amount(50)));

        // Make sure that player 0 won!
        // pot = 18 + 25 = 43 (9 contributed by player 1)
        assert_eq!(
            state
                .active_player()
                .actions()
                .contains(&AuctionPokerAction::Auction(Winner::Player(0))),
            true
        );

        state.update(AuctionPokerAction::Auction(Winner::Player(0)));

        // Should be expecting to get a hole card
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::DealHole(_, 0))));

        state.update(AuctionPokerAction::DealHole(
            Card::new("3c").to_usize().unwrap(),
            0,
        ));

        // Make sure that we go to the BettingRoundStart marker
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundStart));

        state.update(AuctionPokerAction::BettingRoundStart);
        // Check if it's the first player and we're allowed to raise
        assert_eq!(
            state
                .active_player()
                .actions()
                .iter()
                .any(|x| matches!(x, AuctionPokerAction::Raise(_))),
            true
        );
        assert_eq!(state.active_player().player_num() == 1, true);

        state.update(AuctionPokerAction::Check);
        // Check that we marked the player's action with PlayerActionEnd
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::PlayerActionEnd(1))));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Check);

        // Make sure betting round is over
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundEnd));
        state.update(AuctionPokerAction::BettingRoundEnd);

        // Make sure we're in the card dealing round
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::DealCommunity(_))));

        // Turn dealt
        state.update(AuctionPokerAction::DealCommunity(
            Card::new("Qc").to_usize().unwrap(),
        ));

        // Make sure that we have moved on to the next betting round!
        // Check if BettingRoundStart marker is present
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundStart));
        state.update(AuctionPokerAction::BettingRoundStart);

        println!(" Active player {:?}", state.active_player());
        assert_eq!(
            state
                .active_player()
                .actions()
                .iter()
                .any(|x| matches!(x, AuctionPokerAction::Raise(_))),
            true
        );
        assert_eq!(state.active_player().player_num() == 1, true);

        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Check);
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundEnd));
        state.update(AuctionPokerAction::BettingRoundEnd);

        // Make sure we're in the card dealing round
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::DealCommunity(_))));

        // River dealt
        state.update(AuctionPokerAction::DealCommunity(
            Card::new("5c").to_usize().unwrap(),
        ));

        // Check for BettingRoundStart marker
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundStart));
        state.update(AuctionPokerAction::BettingRoundStart);

        // Check if it's the second player and we're allowed to raise
        assert_eq!(
            state
                .active_player()
                .actions()
                .iter()
                .any(|x| matches!(x, AuctionPokerAction::Raise(__))),
            true
        );
        assert_eq!(state.active_player().player_num() == 1, true);

        // Add 2 more to the pot
        // there's now 9 + 9 = 18 contribution in the pot for the losing player
        state.update(AuctionPokerAction::Raise(Amount(2)));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::PlayerActionEnd(1))));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(9)));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| matches!(x, AuctionPokerAction::PlayerActionEnd(0))));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Call);
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::BettingRoundEnd));
        state.update(AuctionPokerAction::BettingRoundEnd);

        assert!(matches!(state.active_player(), ActivePlayer::Terminal(_)));
        if let ActivePlayer::Terminal(deltas) = state.active_player() {
            assert!((deltas[0] - 18.0) < 0.00001); // Player 0 should get all the prize mulah
            assert!(
                (deltas[1] - -18.0) < 0.00001,
                "Player 1 should lose all the prize mulah {:?}",
                deltas
            );
        }
    }

    #[test]
    fn test_can_fold_on_preflop_raise() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(3)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Fold));
    }

    #[test]
    fn test_can_fold_on_flop() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(1)));
        state.update(AuctionPokerAction::Bid(Amount(0)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Auction(Winner::Player(1))));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Fold));
    }

    #[test]
    fn test_can_fold_on_turn() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(1)));
        state.update(AuctionPokerAction::Bid(Amount(0)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Auction(Winner::Player(1))));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(32));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(100)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Fold));
    }

    #[test]
    fn test_can_fold_on_river() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(1)));
        state.update(AuctionPokerAction::Bid(Amount(0)));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Auction(Winner::Player(1))));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(32));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(100)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Fold));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(30));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Fold));
        
    }

    #[test]
    fn test_all_in() {
        // Make sure that all-in works especially when there are asymmetric
        // contributions to the stack
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        // pot is 4
        state.update(AuctionPokerAction::BettingRoundEnd);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(200)));
        state.update(AuctionPokerAction::Bid(Amount(100)));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));
        // pot is 104 with player 1 having 102 contributed
        // so player 1 stack is 400 - 102 = 298
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(100))); // p1 raises 90 more
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Raise(Amount(200))); // p0 raises 100 more
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .contains(&AuctionPokerAction::Raise(Amount(298)))); // p1 should be able to
                                                              //// raise all in even though
                                                              //// they don't have enough
                                                              //// to match p0's raise
        // BUT there should be no other raise sizes
        assert!(!state
            .active_player()
            .actions()
            .iter()
            .filter(|x| matches!(x, AuctionPokerAction::Raise(_)))
            .any(|x| x != &AuctionPokerAction::Raise(Amount(298))));

        state.update(AuctionPokerAction::Raise(Amount(298))); 
    }

    #[test]
    fn test_re_raise_preflop() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(100)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        // Can still raise after raising
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));
        state.update(AuctionPokerAction::Raise(Amount(200)));
    }

    #[test]
    fn test_min_raise() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        // TODO: tests are wrong, should be DeciPercent
        assert!(!state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(17)))));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(18)))));
        state.update(AuctionPokerAction::Raise(Amount(100)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(!state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(189)))));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(190)))));
        // Can still raise after raising
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));
        state.update(AuctionPokerAction::Raise(Amount(200)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(!state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(299)))));
        assert!(!state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(Amount(300)))));
    }

    #[test]
    fn test_can_always_call_a_legal_raise() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .contains( &AuctionPokerAction::Call));
        state.update(AuctionPokerAction::Raise(Amount(100)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .contains( &AuctionPokerAction::Call));
        state.update(AuctionPokerAction::Raise(Amount(200)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .contains( &AuctionPokerAction::Call));
    }
    #[test]
    fn test_cannot_raise_more_than_aggression_limit() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);

        assert_eq!(AGGRESSION_LIMIT, 6, "Test is invalid");

        state.update(AuctionPokerAction::Raise(Amount(10)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Raise(Amount(20)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(30)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::Raise(Amount(40)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Raise(Amount(50)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));
        state.update(AuctionPokerAction::Raise(Amount(60)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));

        assert!(!state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));

    }

    #[test]
    fn test_cannot_raise_at_0_stack() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);

        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(398)));
        state.update(AuctionPokerAction::Bid(Amount(397)));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));

        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Raise(Amount(1)));
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        state.update(AuctionPokerAction::BettingRoundEnd);

        state.update(AuctionPokerAction::DealCommunity(9));
        state.update(AuctionPokerAction::BettingRoundStart);
        assert!(state
            .active_player()
            .actions()
            .iter()
            .all(|x| !matches!(x, AuctionPokerAction::Raise(_))));
    }

    #[test]
    fn test_card_coherence() {
        let card_str = "9d";
        let card_interpreted = Card::from_index(Card::new(card_str).to_usize().unwrap());
        assert_eq!(card_interpreted, Card::new(card_str));
        assert_eq!(card_interpreted.to_string().unwrap(), card_str.to_owned());
    }
    #[test] 
    fn test_check_raise_fold() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);

        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(50)));
        state.update(AuctionPokerAction::Bid(Amount(20)));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));

        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));
        state.update(AuctionPokerAction::Raise(Amount(20)));
        state.update(AuctionPokerAction::PlayerActionEnd(0));
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Fold)));
    }

    #[test] 
    fn test_check_raise_flop() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::BettingRoundEnd);

        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::AuctionStart);
        state.update(AuctionPokerAction::Bid(Amount(50)));
        state.update(AuctionPokerAction::Bid(Amount(20)));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));

        state.update(AuctionPokerAction::BettingRoundStart);
        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::PlayerActionEnd(1));
        println!("{:?}", state);
        assert!(state
            .active_player()
            .actions()
            .iter()
            .any(|x| matches!(x, AuctionPokerAction::Raise(_))));
        state.update(AuctionPokerAction::Raise(Amount(20)));
    }

    #[test]
    fn percent() {
        let amount = Amount(100);
        assert_eq!(1000, amount.to_percent(100));
        assert_eq!(2000, amount.to_percent(50));
        let percent = DeciPercent(1000);
        assert_eq!(100, percent.to_amount(100));
        assert_eq!(50 , DeciPercent(Amount(50).to_percent(50)).to_amount(50));
    }
}
