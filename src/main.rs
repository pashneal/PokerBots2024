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

use crate::strategy::PolicyMap;
use std::sync::Arc;

pub type Utility = f32;


pub fn main() -> () {

    let g = Game::<KuhnPokerAction, KuhnPokerState>::new();

    let mut mcp = MCCFRParallel::<KuhnPokerAction, KuhnPokerState>::new(10);
    mcp.run_iterations(100_000, 0.6);

    mcp.write_to("kuhn");
}
