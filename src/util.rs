use crate::constants::*;
use crate::game_logic::action::*;
use crate::implementations::goofspiel::GoofspielAction;

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

pub fn top_values() -> Filter<GoofspielAction> {
    is(GoofspielAction(4)).or(is(GoofspielAction(5)))
}

pub fn bottom_values() -> Filter<GoofspielAction> {
    not(top_values())
}

pub fn is<T: Filterable>(value: T) -> Filter<T> {
    Filter::new(value)
}

pub fn not<T: Filterable>(value: Filter<T>) -> Filter<T> {
    Filter::not(value)
}

pub fn card_range(range: StdRange<usize>) -> Filter<GoofspielAction> {
    Filter::range(range)
}
