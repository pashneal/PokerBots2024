use crate::game_logic::action::{Action, ActionIndex};
use crate::game_logic::action::GameMapper;
use crate::game_logic::state::State;
use crate::{Categorical, Game};
use dashmap::DashMap;
use crossbeam::atomic::AtomicCell; 


use std::fs::File;
use std::io::Write;

pub type CondensedInfoSet = u64;
pub type PolicyDistribution = Vec<f32>;
pub type RegretDistribution = Vec<f32>;
pub type PolicyMap = DashMap<CondensedInfoSet, PolicyDistribution>;
pub type RegretMap = DashMap<CondensedInfoSet, RegretDistribution>;

#[derive(Clone, Debug)]
pub struct RegretStrategy{
    //iterations: AtomicCell<usize>,
    policy_map: PolicyMap,
    regret_map: RegretMap,
}

impl Default for RegretStrategy {
    fn default() -> Self {
        RegretStrategy {
            //iterations: 0,
            policy_map : DashMap::new(),
            regret_map : DashMap::new(),
        }
    }
}

impl RegretStrategy {

    pub fn regrets(&self, information_set: &CondensedInfoSet) -> Option<RegretDistribution> {
        // Hmmmmm??
        // TODO: speeeeeeeeeeeeeeeeed get rid of the clone somehow
        self.regret_map.get(information_set).map(|r| (*r).clone()).map(|v| Vec::from(v))
    }
    pub fn policy(&self, information_set: &CondensedInfoSet) -> Option<PolicyDistribution> {
        // Hmmmmm??
        // TODO: speeeeeeeeeeeeeeeeed, get rid of the clone somehow
        self.policy_map.get(information_set).map(|r| (*r).clone()).map(|v| Vec::from(v))
    }
    pub fn save_table_json<A : Action>(&self, file_name: &str, action_mapper: &GameMapper<A>) {
        let mut file = File::create(file_name).unwrap();
        let mut table = Vec::new();
        println!("Saving table to {}", file_name);
        for reference in self.policy_map.iter() {
            let (information_set, strategy) = reference.pair();

            // Optimization: only if there is a non-zero value in the strategy, add it to the table
            if strategy.iter().all(|&x| x < 0.0001) {
                continue;
            }
            // Optimization: If there is only one choice in the strategy, don't add it to the table
            if strategy.len() == 1 {
                continue;
            }
            let strategy = normalized(strategy.clone());
            table.push((information_set.clone(), strategy.clone()));
        }
        let json = serde_json::to_string(&table).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    ///[Neal] Update the policy distribution of an information set based on the regrets
    /// and current strategy
    pub fn update(
        &self,
        info_set: CondensedInfoSet,
        d_reg: Option<&[f32]>, // [Neal] Observed current regrets at a terminal history
        d_strat: Option<&[f32]>, // [Neal] Observed current strategy at a terminal history TODO: ?
    ) {
        let len = d_reg
            .or(d_strat)
            .expect("Pass at least one of d_reg, d_strat to update")
            .len();
        if let Some(d) = d_strat {

            let entry = self.policy_map.entry(info_set.clone());
            let mut val = entry.or_insert_with(|| vec![0.0; len]);
            if len != d.len() {
                panic!("Passed d_reg and d_strat must have same length.")
            }
            for (ve, de) in val.iter_mut().zip(d) {
                *ve += de;
            }
        }
        if let Some(d) = d_reg {
            let entry = self.regret_map.entry(info_set.clone());
            let mut val = entry.or_insert_with(|| vec![0.0; len]);
            for (ve, de) in val.iter_mut().zip(d) {
                *ve += de;
            }
        }
    }

    pub fn size(&self) -> usize {
        self.policy_map.len()
    }
}

pub fn normalized(v: Vec<f32>) -> Vec<f32> {
    let mut sum = 0.0;
    for e in &v {
        sum += e;
    }
    let mut normalized = Vec::new();
    for e in v {
        // Round to 4 decimal places
        let e = e / sum;
        let e = (e * 10000.0).round() / 10000.0;
        normalized.push(e);
    }
    normalized
}
