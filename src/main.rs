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


use crate::game_logic::strategy::blueprint::*;

pub type Utility = f32;

pub fn main() -> () {
    
    //let time = std::time::Instant::now();
    //let strat = BlueprintStrategy::load_from_json("auction_poker_p0.json", "auction_poker_p1.json");
    //strat.save_bincode("auction_poker.bp"); 
    //let strat = BlueprintStrategy::load_bincode("auction_poker.bp");
    //loop {
        
    //}


    
    let mut mcp = MCCFRParallel::<AuctionPokerAction, AuctionPokerState>::new(11);
    mcp.run_iterations(1100, 0.2);
    mcp.write_to("auction_poker");
}
