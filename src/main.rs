mod action;
mod constants;
mod distribution;
mod game;
pub mod goofspiel;
mod history;
pub mod kuhn_poker;
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
use crate::action::*;
use crate::goofspiel::{GoofspielAction, GoofspielState};
use crate::kuhn_poker::*;
use crate::util::*;
use rand::{rngs::SmallRng, SeedableRng};

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
    //let g = Game::<GoofspielAction, GoofspielState>::new();

    //let mut mc = MCCFR::new(g);
    //mc.with_game_mapper(test_abstractions());
    //let mut rng = SmallRng::seed_from_u64(2);
    //mc.run_iterations(1000000, 0.6, &mut rng);

    //mc.write_to("goofspiel");

    let g = Game::<KuhnPokerAction, KuhnPokerState>::new();

    let mut mc = MCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(2);
    mc.run_iterations(1000000, 0.6, &mut rng);

    mc.write_to("kuhn_poker");
}

