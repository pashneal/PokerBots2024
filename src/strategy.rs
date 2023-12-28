use crate::action::Action;
use crate::action::{GameMapper, HotEncoding, IntoHotEncoding};
use crate::state::State;
use crate::{ActionIndex, Categorical, Game};
use std::collections::HashMap;

use std::fs::File;
use std::io::Write;

pub type InformationSet<A> = Vec<A>;
pub type StrategyDistribution = Vec<f64>;
pub type RegretDistribution = Vec<f64>;
pub type Mapping<A> = HashMap<InformationSet<A>, (StrategyDistribution, RegretDistribution)>;

#[derive(Clone, Debug)]
pub struct RegretStrategy<A: Action> {
    pub updates: usize,
    pub iterations: usize,
    information_sets: Mapping<A>,
}

impl<A: Action> Default for RegretStrategy<A> {
    fn default() -> Self {
        RegretStrategy {
            iterations: 0,
            updates: 0,
            information_sets: Default::default(),
        }
    }
}

impl<A: Action> RegretStrategy<A> {
    pub fn get(
        &self,
        info_set: &InformationSet<A>,
    ) -> Option<&(StrategyDistribution, RegretDistribution)> {
        self.information_sets.get(info_set)
    }
    pub fn save_table_json(&self, file_name: &str) {
        let mut file = File::create(file_name).unwrap();
        let mut table = Vec::new();
        for (information_set, (strategy, _)) in &self.information_sets {
            let info_set = to_encodings(information_set.clone());
            let info_set = to_int(info_set);
            let info_set = to_binary(info_set);

            // If there is a non-zero value in the strategy, add it to the table
            if strategy.iter().all(|&x| x < 0.0001) {
                continue;
            }
            // If  there is only one choice in the strategy, don't add it to the table
            if strategy.len() == 1 {
                continue;
            }
            let strategy = normalized(strategy.clone());
            table.push((info_set, strategy.clone()));
        }
        let json = serde_json::to_string(&table).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    ///[Neal] Update the policy distribution of an information set based on the regrets
    /// and current strategy
    pub fn update(
        &mut self,
        info_set: Vec<A>,
        d_reg: Option<&[f64]>, // [Neal] Observed current regrets at a terminal history
        d_strat: Option<&[f64]>, // [Neal] Observed current strategy at a terminal history TODO: ?
    ) {
        self.updates += 1;
        let len = d_reg
            .or(d_strat)
            .expect("Pass at least one of d_reg, d_strat to update")
            .len();
        let entry = self.information_sets.entry(info_set);
        let val = entry.or_insert_with(|| (vec![0.0; len], vec![0.0; len]));
        if let Some(d) = d_strat {
            if len != d.len() {
                panic!("Passed d_reg and d_strat must have same length.")
            }
            for (ve, de) in val.0.iter_mut().zip(d) {
                *ve += de;
            }
        }
        if let Some(d) = d_reg {
            for (ve, de) in val.1.iter_mut().zip(d) {
                *ve += de;
            }
        }
    }
}

pub fn to_encodings(v: Vec<impl IntoHotEncoding>) -> Vec<HotEncoding> {
    let mut encodings = Vec::new();
    for e in v {
        encodings.push(e.encoding());
    }
    encodings
}

pub fn to_int(v: Vec<HotEncoding>) -> Vec<Vec<i32>> {
    let mut all = Vec::new();
    for e in v {
        let mut int_vec = Vec::new();
        for b in e {
            int_vec.push(b as i32);
        }
        all.push(int_vec);
    }
    all
}

pub fn to_binary(v: Vec<Vec<i32>>) -> Vec<i32> {
    let mut all = Vec::new();
    // Convert to binary number based on where the 1 is
    for e in v {
        let mut binary = 0;
        for (i, b) in e.iter().enumerate() {
            if *b == 1 {
                binary += 2_i32.pow(i as u32);
            }
        }
        all.push(binary);
    }
    all
}

pub fn normalized(v: Vec<f64>) -> Vec<f64> {
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
