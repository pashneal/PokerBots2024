use std::collections::BTreeMap;
use crate::game_logic::visibility::History;
use crate::implementations::auction::*;
use crate::game_logic::action::*;
use crate::game_logic::game::*;
use crate::game_logic::visibility::*;

use crate::game_logic::strategy::CondensedInfoSet;
use crate::game_logic::strategy::PolicyDistribution;
use crate::game_logic::strategy::RegretDistribution;
use crate::game_logic::strategy::RegretMap;
use crate::game_logic::strategy::PolicyMap;

use crate::constants::*;

use std::ops::Bound::Included;


use serde::{Deserialize, Serialize};
use std::io::{Write, Read};

const MAX_POLICY_LENGTH : usize = 90;  // The maximum number of items in a policy distribution

const MAX_VALUE_SIZE_BITS : usize = 10; // The maximum number of bits needed to represent

const COMPRESSED_SIZE : usize = 10;     

const CHOSEN_COMPRESSION_BITS : usize = 128;  // The bits in the
                                              // CondensedPolicyDistribution that we chose
const MAX_FIT : usize = CHOSEN_COMPRESSION_BITS / MAX_VALUE_SIZE_BITS;

const ARRAY_SIZE : usize =  MAX_POLICY_LENGTH / MAX_FIT;

const FAIL_CUTOFF : i32  = 1000;
                                            
type CondensedPolicyDistribution = [u128; ARRAY_SIZE];


#[derive(Clone, Debug, Copy)]
pub enum FitFunction {
    Range(i32, i32),
    Difference,
    Exact,
}

#[derive(Clone, Debug, Default)]
pub struct Evaluator  {
    pub preflop : Vec<FitFunction>,
    pub auction : Vec<FitFunction>,
    pub flop_onwards: Vec<FitFunction>
}





impl Evaluator {

    fn loss ( target : &History, test : &History, functions : &Vec<FitFunction> ) -> i32 {
        let target = target.0.clone();
        let test = test.0.clone();
        let mut loss = 0;
        for ((&function, &target), &test) in functions.iter().zip(target.iter()).zip(test.iter()) {
            let dl = match function {
                FitFunction::Range( _ , _) => {
                    //TODO: can make it nonlinear loss 
                    (test as i32- target as i32).abs()
                }
                FitFunction::Exact => { 
                    match test == target {
                        true => 0,
                        false => FAIL_CUTOFF,
                    }
                }
                FitFunction::Difference => {
                    (test as i32- target as i32).abs()
                }
            };

            loss += dl;
        };

        loss
    }

    fn get_min_max( target_value : u8 , function : FitFunction) -> (u8, u8) {
        match function {
            FitFunction::Range(pos_delta , neg_delta) => {
                let value = target_value as i32;
                let max_index = Feature::max_index() as i32;
                let max = (value + pos_delta).min(max_index);
                let min = (value + neg_delta).max(0);
                (min as u8, max as u8)
            }

            FitFunction::Exact => {
                (target_value, target_value)
            }

            FitFunction::Difference => {
                (0, Feature::max_index() as u8)
            }
        }
    }
    fn get_best(&self, map : &BTreeMap<CondensedInfoSet, CondensedPolicyDistribution>, target : CondensedInfoSet) -> Option<CondensedInfoSet> {

        let history : History = target.clone().into();
        let history  = history.0;
        let round : Round = (history[0] as usize).into();

        let evaluator = match round {
            Round::PreFlop => self.preflop.clone(),
            Round::Auction => self.auction.clone(),
            Round::Flop | Round::Turn | Round::River => self.flop_onwards.clone(),
        };

        debug_assert_eq!(evaluator.len(), history.len(), "History does not match the evaluation
        array");

        let ranges = history.iter().zip(evaluator.iter()).map(|(x, func)| Evaluator::get_min_max( *x , *func));

        let min_values :  Vec<u8> = ranges.clone().map( |(min, _)|  min).collect();
        let max_values :  Vec<u8> = ranges.clone().map( |(_, max)|  max).collect();

        println!("Min values: {:?}", min_values);
        println!("Max values: {:?}", max_values);

        let min_info_set = History(min_values).into_condensed();
        let max_info_set = History(max_values).into_condensed();

        let possible_values = map.range((Included(min_info_set) , Included(max_info_set)));

        let mut min_loss = FAIL_CUTOFF;
        let mut min_key = None;

        let target : History = target.into();
        for (&key, _) in possible_values {
            let test : History = key.into();
            let loss = Evaluator::loss(&target, &test, &evaluator) ;
            if loss < min_loss{
                min_loss = loss;
                min_key = Some(key);
            }
        };

        println!("[STATS] For the curious, min loss for this policy: {:?}", min_loss);
        min_key


    }
}


pub fn compress(value : f32) -> u128 {
    let result = (value * 999.0) as u128;
    result
}

pub fn decompress(value : u128) -> f32 {
    let max_value = 999.0;
    let result = value as f32 / max_value; 
    result
}

pub fn compress_policy(policy : &PolicyDistribution) -> CondensedPolicyDistribution {
    let mut result = [0; ARRAY_SIZE];
    for (i, chunks) in policy.chunks(MAX_FIT).enumerate() {
        let mut total = 0;
        for (_, &value) in chunks.iter().rev().enumerate() {
            let value = compress(value);
            total *= 1000;
            total += value;
        }
        result[i] = total;
    }
    result
}

pub fn decompress_policy(policy : &CondensedPolicyDistribution) -> PolicyDistribution {
    let mut result = Vec::new();
    for &chunk in policy.iter() {
        let mut chunk = chunk;
        for _ in 0..MAX_FIT {
            let value = chunk % 1000;
            chunk = chunk / 1000;
            result.push(decompress(value));
        }
    }
    result
}

pub fn analyze_policy(info_set: CondensedInfoSet , policy : &PolicyDistribution) {

    //TODO: need 
    
    let history : History  = info_set.into();
    let ev = history.0[1];
    if history.0.len() <6 || ev != 70{
        return;
    }
    let v : Vec<f32> = policy.into_iter().map(|x| if *x < 0.02 { 0.0 } else { *x }).collect();
    println!("{:?}",history);
    println!("{:?}", v);
    for (i, &value) in v.iter().enumerate() {
        let i : AuctionPokerAction  = (i as ActionIndex).into();
        if value > 1e-3 {
            print!("{:?}: {:?} \n", i, value);
        }
    }
    println!();

}

#[derive(Clone, Debug)]
pub struct BlueprintStrategy {
    policies : Vec<BTreeMap<CondensedInfoSet, CondensedPolicyDistribution>>,
    evaluator : Evaluator,

}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct SavedStrategy {
    pub information : Vec<(CondensedInfoSet, PolicyDistribution)>,
}

pub fn load(file_name : &str) -> SavedStrategy {
    let file = std::fs::File::open(file_name).unwrap();
    let reader = std::io::BufReader::new(file);
    let strategy : SavedStrategy = serde_json::from_reader(reader).unwrap();
    strategy
}

impl BlueprintStrategy {
    pub fn load_from_json(player0_file : &str , player1_file : &str) -> BlueprintStrategy {
        println!("Loading player 0 strategy from {}", player0_file);
        let time = std::time::Instant::now();
        let strategy0 = load(player0_file);
        println!("Time to load player 0 {:?}", time.elapsed());

        println!("Loading player 1 strategy from {}", player1_file);
        let time = std::time::Instant::now();
        let strategy1 = load(player1_file);
        println!("Time to load player 1 {:?}", time.elapsed());

        let mut policy0 = BTreeMap::new();
        let mut policy1 = BTreeMap::new();

        println!("Merging strategies");
        let time = std::time::Instant::now();
        for (info_set, policy) in strategy0.information {
            let history : History = info_set.clone().into();
            policy0.insert(info_set, compress_policy(&policy));
        }
        println!("Time to merge (0) {:?}", time.elapsed());
        for (info_set, policy) in strategy1.information {
            policy1.insert(info_set, compress_policy(&policy));
        }
        println!("Time to merge (1) {:?}", time.elapsed());

        BlueprintStrategy {
            policies : vec![policy0, policy1],
            evaluator : Evaluator::default(),
        }
    }


    pub fn with_evaluator(self, evaluator : Evaluator) -> BlueprintStrategy{
        BlueprintStrategy {
            policies : self.policies,
            evaluator,
        }
    }

    pub fn save_bincode(&self, file_name : &str) {
        println!("Saving strategy to {}", file_name);
        let file = std::fs::File::create(file_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        
        let time = std::time::Instant::now();
        let vecs: Vec<Vec<(CondensedInfoSet, CondensedPolicyDistribution)>> = self.policies.iter().map(|policy| {
            policy.iter().map(|(info_set, policy)| {
                (*info_set, *policy)
            }).collect()
        }).collect();
        println!("Time to convert {:?}", time.elapsed());

        let time = std::time::Instant::now();
        bincode::serialize_into(writer, &vecs).unwrap();
        println!("Time to save {:?}", time.elapsed());
    }

    pub fn load_bincode(file_name : &str) -> BlueprintStrategy {
        println!("Loading strategy from {}", file_name);
        let time = std::time::Instant::now();
        let file = std::fs::File::open(file_name).unwrap();
        let reader = std::io::BufReader::new(file);
        let strategy : Vec<Vec<(CondensedInfoSet, CondensedPolicyDistribution)>> = bincode::deserialize_from(reader).unwrap();
        println!("Time to load {:?}", time.elapsed());
        let mut policies = Vec::new();
        let time = std::time::Instant::now();
        for player in strategy {
            let mut policy = BTreeMap::new();
            for (info_set, policy_distribution) in player {
                policy.insert(info_set, policy_distribution);
            }
            policies.push(policy);
        }
        println!("Time to convert {:?}", time.elapsed());
        BlueprintStrategy {
            policies,
            evaluator : Evaluator::default(),
        }
    }

    fn normalize_policy(&self,  condensed_policy: &Option<CondensedPolicyDistribution>) -> Option<Vec<(ActionIndex, f32)>> {
        let policy = match condensed_policy {
            Some(policy) => decompress_policy(policy),
            None => panic!("No policy found!"),
        };

        let filtered_policy : Vec<(ActionIndex, f32)>= policy.iter().enumerate().filter_map( | (action_index, probability) |{
            match *probability > BLUEPRINT_CUTOFF   {
                true => Some((action_index as ActionIndex, *probability)),
                false => None
            }
        }).collect();

        if filtered_policy.len() == 0 {
            return None;
        }

        let probabilities : Vec<f32> = filtered_policy.iter().map( |(_ , prob)| *prob).collect();
        let sum : f32 = probabilities.iter().sum();

        if sum < 1e-5 {
            return None;
        }

        let normalized : Vec<(ActionIndex, f32)>  = filtered_policy.iter().map(| (action_index, probability) | {
            (*action_index, probability / sum)
        }).collect();

        
        Some(normalized)
    }

    /// Returns a probability distribution over
    /// chosen ActionIndex given a current game
    /// 
    /// Uses a collection of evaluator function to determine the best policy for 
    /// an info set that doesn't exist in the current blueprint strategy
    ///
    /// returns None if unable to find a suitable normalized strategy
    pub fn get_best_policy(&self, game: &Game<AuctionPokerAction, AuctionPokerState>, player_num: usize) -> Option<Vec<(ActionIndex, f32)>> {
        let current_info_set = game.get_information_set(player_num);
        let history : History = current_info_set.clone().into();
        println!("Current history set {:?}", history);
        let best_info_set  = self.evaluator.get_best(&self.policies[player_num], current_info_set);
        let policy = best_info_set.map(|info_set| self.policies[player_num][&info_set]);
        self.normalize_policy(&policy)
    }

    /// Returns a probability distribution over
    /// chosen ActionIndex given a current game
    ///
    /// returns None if unable to find a suitable normalized strategy
    pub fn get_exact_policy(&self, game : &Game<AuctionPokerAction, AuctionPokerState>, player_num: usize) -> Option<Vec<(ActionIndex, f32)>> {
        let info_set = game.get_information_set(player_num);
        let condensed_policy = self.policies[player_num].get(&info_set).map(|policy| *policy);
        self.normalize_policy(&condensed_policy)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::auction::*;
    use crate::implementations::auction::RelativeSize::*;
    #[test]
    pub fn test_model_can_give_fitting_suggestions() {
        let mut g = Game::<AuctionPokerAction, AuctionPokerState>::new();
        g.play(&AuctionPokerAction::DealHole(0, 0));
        g.play(&AuctionPokerAction::DealHole(2, 0));
        g.play(&AuctionPokerAction::DealHole(3, 1));
        g.play(&AuctionPokerAction::DealHole(4, 1));
        g.play(&AuctionPokerAction::BettingRoundStart);
        let strategy = BlueprintStrategy::load_bincode("auction_poker.bp");

        let preflop_evaluator = Evaluator {
            preflop : vec![
                FitFunction::Exact, //  Round must be exact
                FitFunction::Exact, //  Ranks must be exact
                FitFunction::Exact, //  Suited must be exact
                FitFunction::Exact, //  Aggression must be exact
                FitFunction::Difference, //  Pot is allowed to be different
            ],
            auction : vec![],
            flop_onwards : vec![],
        };

        let strategy = strategy.with_evaluator(preflop_evaluator);

        let policy = strategy.get_best_policy(&g, 0);
        let policy2 = strategy.get_exact_policy(&g, 0);
        assert!(policy.is_some());
        assert_eq!(policy, policy2, "Best fit policy should be the same as exact policy");
    }

    #[test]
    pub fn test_model_can_give_initial_suggestions() {
        let mut g = Game::<AuctionPokerAction, AuctionPokerState>::new();
        g.play(&AuctionPokerAction::DealHole(0, 0));
        g.play(&AuctionPokerAction::DealHole(2, 0));
        g.play(&AuctionPokerAction::DealHole(3, 1));
        g.play(&AuctionPokerAction::DealHole(10, 1));
        g.play(&AuctionPokerAction::BettingRoundStart);
        let strategy = BlueprintStrategy::load_bincode("auction_poker.bp");
        let policy = strategy.get_exact_policy(&g, 0);
        assert!(policy.is_some());
        println!("For the curious, the policy for a pair of Aces: {:?}", policy);
        let preflop_evaluator = Evaluator {
            preflop : vec![
                FitFunction::Exact, //  Round must be exact
                FitFunction::Exact, //  Ranks must be exact
                FitFunction::Exact, //  Suited must be exact
                FitFunction::Exact, //  Aggression must be exact 
                FitFunction::Difference, //  Pot is allowed to be different
            ],
            auction : vec![],
            flop_onwards : vec![],
        };
        let strategy = strategy.with_evaluator(preflop_evaluator);
        let bet_size = Amount(40);
        g.play(&AuctionPokerAction::Raise(bet_size.clone()));
        g.play(&AuctionPokerAction::PlayerActionEnd(0));
        let policy = strategy.get_best_policy(&g, 1);
        assert!(policy.is_some());
        println!("For the curious, the policy in response to Raise({:?})\n {:?}", bet_size, policy);


    }
    #[test]
    pub fn test_model_knows_when_to_fold() {
        // Assumes that there is a model named "auction_poker.bp" in the current directory
        let strategy = BlueprintStrategy::load_bincode("auction_poker.bp");

        let mut folded = 0;
        for policy in strategy.policies.clone() {
            for (info_set, policy) in policy.iter() {
                let decompressed = decompress_policy(policy);
                let history : History = (*info_set).into();
                let round = history.0[0];
                if round > 1 {
                    let fold_freq = decompressed[AuctionPokerAction::Fold.index() as usize];
                    if fold_freq > 0.80 {
                        folded += 1;
                    }
                }
            }
        }
        assert!(folded > 0 , "There should be at least some nodes with very high folding frequency");
    }


    #[test]
    pub fn decompress_compress() {
        let mut policy = vec![0.0; 40];
        policy[0] = 0.5;
        policy[1] = 0.5;
        policy[9] = 0.5;
        let compressed = compress_policy(&policy);
        let decompressed = decompress_policy(&compressed);
        assert!(policy[0]  - decompressed[0] < 1e-3);
        assert!(policy[1]  - decompressed[1] < 1e-3);
        assert!(policy[9]  - decompressed[9] < 1e-3);
    }
}
