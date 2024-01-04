use crate::game_logic::action::*;
use crate::game_logic::state::{ActivePlayer, State};
use crate::game_logic::visibility::Observation;

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
        None
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
        None
    }
}

impl Card {
    fn new(s: String) -> Self {
        let mut chars = s.chars();
        let value = Value::new(chars.next().unwrap().to_string());
        let suit = Suit::new(chars.next().unwrap().to_string());
        Card { value, suit }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hand {
    pub cards: (Card, Card),
}

impl Parsable for Hand {
    fn to_string(&self) -> Option<String> {
        // Sort the cards so that the order is always the same
        let mut cards = vec![self.cards.0.clone(), self.cards.1.clone()];
        cards.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        let s = format!(
            "{}{}",
            cards[0].to_string().unwrap(),
            cards[1].to_string().unwrap()
        );
        Some(s)
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Hand {
    fn new(s: String) -> Self {
        let mut chars = s.chars();
        let card1 =
            Card::new(chars.next().unwrap().to_string() + &chars.next().unwrap().to_string());
        let card2 =
            Card::new(chars.next().unwrap().to_string() + &chars.next().unwrap().to_string());
        Hand {
            cards: (card1, card2),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TexasHoldEmAction {
    Fold,
    Call,
    Check,
    Deal(Hand),
    Raise(usize, usize), // Pot size, nearest whole percent
}


impl Parsable for TexasHoldEmAction {
    fn to_string(&self) -> Option<String> {
        None
    }

    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Into<ActionIndex> for TexasHoldEmAction {
    fn into(self) -> ActionIndex {
        todo!();
    }
}

impl From<ActionIndex> for TexasHoldEmAction {
    fn from(index: ActionIndex) -> Self {
        todo!();
    }
}

impl Filterable for TexasHoldEmAction {}
impl Action for TexasHoldEmAction {}

#[derive(Debug, Clone)]
pub struct TexasHoldEmState {
    cards: Vec<u32>,
    players_cards: [Option<u32>; 2],
    active_player: ActivePlayer<TexasHoldEmAction>,
}

impl TexasHoldEmState {
    fn dealer(cards: Vec<u32>) -> ActivePlayer<TexasHoldEmAction> {
        todo!()
    }

    /// One of the two players folded 
    fn folded(&self, delta: f32, player_num: usize) -> ActivePlayer<TexasHoldEmAction> {
        todo!()
    }

    /// The game is over, determine the winner
    fn showdown(&self, delta: f32) -> ActivePlayer<TexasHoldEmAction> {
        todo!()
    }
}

impl State<TexasHoldEmAction> for TexasHoldEmState {
    fn new() -> Self {
        todo!()
    }

    fn get_observation(&self, action: &TexasHoldEmAction) -> Observation<TexasHoldEmAction> {
        unimplemented!()
    }

    fn active_player(&self) -> ActivePlayer<TexasHoldEmAction> {
        return self.active_player.clone();
    }

    fn update(&mut self, action: TexasHoldEmAction) {
        match action {
            TexasHoldEmAction::Fold => { unimplemented!() }
            TexasHoldEmAction::Call => { unimplemented!() }
            TexasHoldEmAction::Check => { unimplemented!() }
            TexasHoldEmAction::Deal(x) => { unimplemented!() }
            TexasHoldEmAction::Raise(pot_size, percent) => { unimplemented!() }
        }
    }
}
