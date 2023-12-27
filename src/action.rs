use crate::constants::*;
use crate::ActionIndex;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Range as StdRange;

/// Represents a possible action or observation in the game
/// from a limited number of choices, must have exactly one boolean set to true
pub type HotEncoding = Vec<bool>;

pub trait IntoHotEncoding {
    fn encoding(self) -> HotEncoding;
}



/// generic function time!!! look how readable the resulting syntax is!!!
/// *swoon*
///
/// let hundred_bet = is(Bet(100))
///                 .or(is(Bet(200)))
///                 .or(is(Bet(300)));
///
/// let disjoint_bet = bet_range(0..100)
///                 .or(bet_range(200..300))
///                 .or(bet_range(400..500));
///
/// let suited_kings = suited().and(kings());
/// let not_suited_kings = not(suited_kings);
///
///
/// PRETTY!!!

impl Parsable for u32 {
    fn to_string(&self) -> Option<String> {
        None
    }
    fn to_usize(&self) -> Option<usize> {
        Some(*self as usize)
    }
}

impl Filterable for u32 {}

pub fn top_values() -> Filter<u32> {
    is(4).or(is(5))
}

pub fn bottom_values() -> Filter<u32> {
    not(top_values())
}


pub fn kings() -> Filter<PokerAction> {
    Filter::regex(r"K.K.")
}


pub fn suited() -> Filter<PokerAction> {
    Filter::regex(r".(.).(\1)")
}

pub fn is<T : Filterable>(value : T) -> Filter<T> {
   Filter::new(value) 
}

pub fn not<T : Filterable>(value : Filter<T>) -> Filter<T> {
    Filter::not(value)
}

pub fn bet_range(range : StdRange<usize>) -> Filter<PokerAction> {
    Filter::range(range)
}

pub trait Action: Clone + Debug + Hash + PartialEq + Eq + IntoHotEncoding {}

/// Some default implementations to get us situated for goofspiel impl
impl Action for i32 {}
impl Action for u32 {}

pub type ActionFilter<A> = (Filter<A>, A);

#[derive(Debug, Clone)]
pub struct ActionMapper<A: Filterable> {
    filters: Vec<ActionFilter<A>>,
}

impl<A: Filterable> ActionMapper<A> {
    pub fn new() -> Self {
        ActionMapper {
            filters: Vec::new(),
        }
    }
    pub fn add_filter(&mut self, filter: Filter<A>, action: A) {
        self.filters.push((filter, action));
    }
    pub fn map(&self, action: A) -> A {
        for (filter, mapped_action) in &self.filters {
            if filter.accepts(&action) {
                return mapped_action.clone();
            }
        }
        panic!(
            "No filter matched action, check that your filters span the entire action space!! {:?}",
            action
        );
    }

    pub fn to_index(&self, action: A) -> ActionIndex {
        for (index, (filter, _)) in self.filters.iter().enumerate() {
            if filter.accepts(&action) {
                return index as ActionIndex;
            }
        }
        panic!(
            "No filter matched action, check that your filters span the entire action space!! {:?}",
            action
        );
    }

    pub fn num_groups(&self) -> usize {
        self.filters.len()
    }
}

/// May contain a filter for each depth of the game
/// if no filter is present for a given depth, actions
/// are mapped to themselves
pub struct GameMapper<A: Filterable + Action> {
    depth_specific_maps: Vec<Option<ActionMapper<A>>>,
    recall_depth: usize,
    max_encoding_size: usize,
}

impl<A: Filterable + Action + Into<ActionIndex>> GameMapper<A> {
    ///  Create a GameMapper with no default mapping (passes all actions through)
    ///  recall_depth determines how many states will be
    ///  outputted by a HotEncoding
    fn new(recall_depth: Option<usize>) -> Self {
        let recall_depth = recall_depth.unwrap_or(MAX_GAME_DEPTH);
        GameMapper {
            depth_specific_maps: vec![None; MAX_GAME_DEPTH],
            recall_depth,
            max_encoding_size: HOT_ENCODING_SIZE,
        }
    }
    /// Create a GameMapper with a given default mapping for all depths
    fn from_default(default_map: ActionMapper<A>, recall_depth: Option<usize>) -> Self {
        let recall_depth = recall_depth.unwrap_or(MAX_GAME_DEPTH);
        let encoding_size = default_map.num_groups();
        GameMapper {
            depth_specific_maps: vec![Some(default_map); MAX_GAME_DEPTH],
            recall_depth,
            max_encoding_size: encoding_size,
        }
    }

    /// Create a GameMapper to operate a specific depth of the game
    fn add_map(&mut self, mapper: Option<ActionMapper<A>>, depth: usize) {
        self.depth_specific_maps[depth] = mapper;
        // If there is a mapper, then we need to update the max encoding size
        self.max_encoding_size = 0;
        for mapper in &self.depth_specific_maps {
            match mapper {
                Some(mapper) => {
                    if (mapper.num_groups() > self.max_encoding_size) {
                        self.max_encoding_size = mapper.num_groups();
                    }
                }
                None => self.max_encoding_size = HOT_ENCODING_SIZE,
            }
        }
    }

    fn map(&self, actions: Vec<A>, depth: usize) -> Vec<A> {
        // TODO: figure out what to do if functions map to two different groups
        //       right now it's taking a greedy approach
        let mapper = &self.depth_specific_maps[depth];
        match mapper {
            Some(mapper) => actions.iter().map(|action| mapper.map(action.clone())).collect(),
            None => actions,
        }

    }


    fn encoding(&self, history: &Vec<A>) -> Vec<HotEncoding> {
        debug_assert!(history.len() <= self.recall_depth);
        let mut encodings = Vec::new();
        let max_depth = self.recall_depth;

        // Take last *depth* actions, map them to ActionIndex, and then encode them
        for (index, action) in history.iter().rev().take(max_depth).rev().enumerate() {

            let mapper = &self.depth_specific_maps[index];
            let action_index = match mapper {
                Some(mapper) => mapper.to_index(action.clone()),
                None => action.clone().into()
            };
            let mut encoding = action_index.encoding();
            // Pad the end of any sparser encondings with dummy false values
            encoding.resize(self.max_encoding_size, false);

            encodings.push(encoding);
        }
        encodings
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
    fn to_string(&self) -> Option<String>{
        let string = format!("{}{}", self.value.to_string().unwrap(), self.suit.to_string().unwrap());
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
        let s = format!("{}{}", cards[0].to_string().unwrap(), cards[1].to_string().unwrap());
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
pub enum PokerAction {
    Fold,
    Bet(u32),
    Check,
    Deal(Hand),
}
impl Action for PokerAction {}
impl Parsable for PokerAction {
    fn to_string(&self) -> Option<String> {
        match self {
            PokerAction::Fold => None ,
            PokerAction::Bet(n) => None ,
            PokerAction::Check => None ,
            PokerAction::Deal(hand) => Some(format!("{}", hand.to_string().unwrap())),
        }
    }
    fn to_usize(&self) -> Option<usize> {
        match self {
            PokerAction::Fold => None ,
            PokerAction::Bet(n) => Some(*n as usize),
            PokerAction::Check => None ,
            PokerAction::Deal(hand) => None,
        }
    }
}
impl Filterable for PokerAction{}

/// TODO: this is a dummy implemenation until we have full abstractions
impl IntoHotEncoding for PokerAction {
    fn encoding(self) -> HotEncoding {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Clause<T>
where
    T: Parsable,
{
    pub left: Box<Filter<T>>,
    pub right: Box<Filter<T>>,
}

impl<T> Clause<T>
where
    T: Parsable,
{
    pub fn new(left: Filter<T>, right: Filter<T>) -> Self {
        Clause {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

pub trait Parsable: Clone + Debug + PartialEq {
    fn to_string(&self) -> Option<String>;
    fn to_usize(&self) -> Option<usize>;
}

pub trait Filterable: Parsable {
    fn filter(list: &Vec<Self>, primitive: &Primitive<Self>) -> Vec<Self> {
        match primitive {
            Primitive::Raw(raw) => list.iter().filter(|x| *x == raw).cloned().collect(),
            Primitive::Regex(details) => {
                let re = regex::Regex::new(&details.regex).unwrap();
                list.iter()
                    .filter(|x| match x.to_string() {
                        Some(s) => re.is_match(&s),
                        None => false,
                    })
                    .cloned()
                    .collect()
            }
            Primitive::Range(details) => list
                .iter()
                .filter(|x| match x.to_usize() {
                    Some(n) => details.range.contains(&n),
                    None => false,
                })
                .cloned()
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegexQuery {
    pub regex: String,
}

#[derive(Debug, Clone)]
pub struct RangeQuery {
    pub range: StdRange<usize>,
}

#[derive(Debug, Clone)]
pub enum Primitive<T>
where
    T: Parsable,
{
    Raw(T),
    Regex(RegexQuery),
    Range(RangeQuery),
}

#[derive(Debug, Clone)]
pub enum Filter<T>
where
    T: Parsable,
{
    And(Clause<T>),
    Or(Clause<T>),
    Not(Box<Filter<T>>),
    BaseCase(Primitive<T>),
}

impl<T> Filter<T>
where
    T: Filterable ,
{
    pub fn and(self, other: Filter<T>) -> Self {
        Filter::And(Clause::new(self, other))
    }

    pub fn or(self, other: Filter<T>) -> Self {
        Filter::Or(Clause::new(self, other))
    }

    pub fn new(raw: T) -> Self {
        Filter::BaseCase(Primitive::Raw(raw))
    }

    pub fn regex(regex: &str) -> Self {
        Filter::BaseCase(Primitive::Regex(RegexQuery { regex :regex.to_string()}))
    }

    pub fn range(range: StdRange<usize>) -> Self {
        Filter::BaseCase(Primitive::Range(RangeQuery { range}))
    }

    pub fn not(self) -> Self {
        Filter::Not(Box::new(self))
    }

    pub fn apply_on(&self, list: &Vec<T>) -> Vec<T> {
        match self {
            Filter::And(clause) => {
                let left = clause.left.apply_on(list);
                let right = clause.right.apply_on(list);
                left.into_iter().filter(|x| right.contains(x)).collect()
            }
            Filter::Or(clause) => {
                let left = clause.left.apply_on(list);
                let right = clause.right.apply_on(list);
                left.into_iter().chain(right.into_iter()).collect()
            }
            Filter::Not(filter) => {
                let filtered = filter.apply_on(list);
                list.iter().filter(|x| !filtered.contains(x)).cloned().collect()
            }
            Filter::BaseCase(primitive) => Filterable::filter(list, primitive),
        }
    }

    pub fn accepts(&self, raw: &T) -> bool {
        self.apply_on(&vec![raw.clone()]).len() > 0
    }
}
