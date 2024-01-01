mod action;
mod constants;
mod distribution;
mod game;
pub mod goofspiel;
mod history;
pub mod kuhn_poker;
mod mccfr_parallel;
mod mccfr;
mod state;
mod strategy;
mod util;
mod visibility;

pub use self::action::{HotEncoding, IntoHotEncoding};
pub use self::constants::HOT_ENCODING_SIZE;
pub use self::distribution::Categorical;
pub use self::game::Game;
pub use self::mccfr::MCCFR;
pub use self::mccfr_parallel::MCCFRParallel;
use crate::action::*;
use crate::goofspiel::{GoofspielAction, GoofspielState};
use crate::kuhn_poker::*;
use crate::util::*;
use constants::*;
use crate::strategy::RegretStrategy;
use rand::{rngs::SmallRng, SeedableRng};

use std::sync::Arc;

pub type Utility = f64;

pub fn test_abstractions() -> GameMapper<GoofspielAction> {
    // Set up our filters
    let is_high = card_range(4..=10);
    let is_mid = is(2).or(is(3));

    let very_high = card_range(11..=20);
    let high = is_high.clone();
    let mid = is_mid.clone();
    let is_low = not(mid.or(high).or(very_high)); // You can compose filters!!

    // Action level mapping
    let mut action_mapper = ActionMapper::new();
    action_mapper.add_filter(is_high, 4); // Map all high cards to 4
    action_mapper.add_filter(is_mid, 2); // And so on
    action_mapper.add_filter(is_low, 1);

    // Game level mapping
    // do not specify a recall depth, (has perfect recall)
    // Use our action mapper only for the first depth
    // all other depths will default to passing through the action
    let mut game_mapper = GameMapper::new(None);
    game_mapper.update_depth(Some(action_mapper), 0);

    game_mapper
}

pub fn main() -> () {

    let g = Game::<KuhnPokerAction, KuhnPokerState>::new();

    let mut mcp = MCCFRParallel::<KuhnPokerAction, KuhnPokerState>::new(8);
    mcp.run_iterations(10000000, 0.6);

    mcp.write_to("kuhn_poker");
}
