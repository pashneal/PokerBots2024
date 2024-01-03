mod constants;
mod distribution;
pub mod implementations;
mod algorithm;
mod game_logic;
mod util;

pub use self::constants::HOT_ENCODING_SIZE;
pub use self::distribution::Categorical;
pub use self::game_logic::game::Game;
pub use self::algorithm::mccfr_parallel::MCCFRParallel;
use crate::implementations::kuhn_poker::*;


pub type Utility = f32;


pub fn main() -> () {
    let g = Game::<KuhnPokerAction, KuhnPokerState>::new();
    let mut mcp = MCCFRParallel::<KuhnPokerAction, KuhnPokerState>::new(10);
    mcp.run_iterations(100_000, 0.6);
}
