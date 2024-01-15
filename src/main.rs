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
use crate::implementations::kuhn_poker::*;


use crate::game_logic::strategy::blueprint::*;

pub type Utility = f32;

pub fn main() -> () {
    let mut mcp = MCCFRParallel::<AuctionPokerAction, AuctionPokerState>::new(11);
    mcp.run_iterations(11_000, 0.2);
    mcp.write_to("auction_poker");
    
    //let time = std::time::Instant::now();
    //let strat = BlueprintStrategy::load_from_json("auction_poker_p0.json",
                                                  //"auction_poker_p1.json");
    //strat.save_bincode("auction_poker.bp"); 
    //let strat = BlueprintStrategy::load_bincode("auction_poker.bp");
    ////loop {
        
    //}


    
    //let mut mcp = MCCFRParallel::<KuhnPokerAction, KuhnPokerState>::new(10);
    //mcp.run_iterations(10_000, 0.2);
    //mcp.write_to("kuhn_poker");
    //let strat = BlueprintStrategy::load_from_json("kuhn_poker_p0.json",
                                                  //"kuhn_poker_p1.json");
    //strat.save_bincode("kuhn_poker.bp"); 
    //let strat = BlueprintStrategy::load_bincode("kuhn_poker.bp");
    //loop {
        
    //}
}
