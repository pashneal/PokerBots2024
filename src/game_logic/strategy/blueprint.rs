use std::collections::BTreeMap;
use crate::game_logic::visibility::History;
use crate::implementations::auction::*;
use crate::game_logic::action::*;

use crate::game_logic::strategy::CondensedInfoSet;
use crate::game_logic::strategy::PolicyDistribution;
use crate::game_logic::strategy::RegretDistribution;
use crate::game_logic::strategy::RegretMap;
use crate::game_logic::strategy::PolicyMap;


use serde::{Deserialize, Serialize};
use std::io::{Write, Read};

const MAX_POLICY_LENGTH : usize = 90;  // The maximum number of items in a policy distribution

const MAX_VALUE_SIZE_BITS : usize = 10; // The maximum number of bits needed to represent

const COMPRESSED_SIZE : usize = 10;     

const CHOSEN_COMPRESSION_BITS : usize = 128;  // The bits in the
                                              // CondensedPolicyDistribution that we chose
const MAX_FIT : usize = CHOSEN_COMPRESSION_BITS / MAX_VALUE_SIZE_BITS;

const ARRAY_SIZE : usize =  MAX_POLICY_LENGTH / MAX_FIT;
                                            
type CondensedPolicyDistribution = [u128; ARRAY_SIZE];

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
        for (j, &value) in chunks.iter().rev().enumerate() {
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
    policies : Vec<BTreeMap<CondensedInfoSet, CondensedPolicyDistribution>>, }
//pub fn uncondense_info_set(info_set : &CondensedInfoSet) -> Vec<u8> {
//}

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
            policy0.insert(info_set, compress_policy(&policy));
        }
        println!("Time to merge (0) {:?}", time.elapsed());
        for (info_set, policy) in strategy1.information {
            policy1.insert(info_set, compress_policy(&policy));
        }
        println!("Time to merge (1) {:?}", time.elapsed());

        BlueprintStrategy {
            policies : vec![policy0, policy1],
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
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_model_knows_when_to_fold() {
        // Assumes that there is a model named "auction_poker.bp" in the current directory
        let strategy = BlueprintStrategy::load_bincode("auction_poker.bp");

        for policy in strategy.policies {
            for (info_set, policy) in policy.iter() {
                let decompressed = decompress_policy(policy);
                let history : History = (*info_set).into();
                let round = history.0[0];
                let ev = history.0[1];

                // Only look at flop
                if round != 2 {
                    continue;
                }
                if ev > 5 {
                    continue;
                }
                let fold_freq = decompressed[AuctionPokerAction::Fold.index() as usize];
                assert!( fold_freq > 0.50,
                        "Model should fold with higher frequency (at flop) when EV is very low Found a fold frequency of {}", fold_freq)
            }
            for (info_set, policy) in policy.iter() {
                let decompressed = decompress_policy(policy);
                let history : History = (*info_set).into();
                let round = history.0[0];
                // Only look at the river
                if round != 4 {
                    continue;
                }
                let ev = history.0[1];
                if ev != 100 {
                    continue;
                }
                let fold_freq = decompressed[AuctionPokerAction::Fold.index() as usize];
                assert!(fold_freq < 0.05,
                        "Model should fold with very low frequency (at river) when EV is very low, Found a fold frequency of {}", fold_freq);
            }
        }
    }
}
