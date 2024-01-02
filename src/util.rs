use crate::action::*;
use crate::constants::*;
use crate::goofspiel::GoofspielAction;

/// Example usage:
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

pub fn is<T: Filterable>(value: T) -> Filter<T> {
    Filter::new(value)
}

pub fn not<T: Filterable>(value: Filter<T>) -> Filter<T> {
    Filter::not(value)
}

pub fn bet_range(range: StdRange<usize>) -> Filter<PokerAction> {
    Filter::range(range)
}

pub fn card_range(range: StdRange<usize>) -> Filter<GoofspielAction> {
    Filter::range(range)
}
