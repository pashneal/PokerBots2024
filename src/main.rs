mod algorithm;
mod constants;
mod distribution;
mod eval;
mod game_logic;
pub mod implementations;
mod util;

pub use self::algorithm::mccfr_parallel::MCCFRParallel;
pub use self::constants::HOT_ENCODING_SIZE;
pub use self::distribution::Categorical;
pub use self::game_logic::game::Game;
use crate::implementations::auction::*;

pub type Utility = f32;

pub fn main() -> () {
    let g = Game::<AuctionPokerAction, AuctionPokerState>::new();
    let mut mcp = MCCFRParallel::<AuctionPokerAction, AuctionPokerState>::new(1);
    mcp.run_iterations(1, 0.01);
    // TODO: current bug is that 
    // strategies are imperfect and we need 
    // a why to get the best response
    //
}
