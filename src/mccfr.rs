use crate::action::Action;
use crate::action::{GameMapper, HotEncoding, IntoHotEncoding};
use crate::constants::MAX_GAME_DEPTH;
use crate::state::{ActivePlayer, State};
use crate::strategy::*;
use crate::{Categorical, Game};
use hashbrown::HashMap;
use rand::Rng;
use serde_json::json;
use std::collections::HashMap as SerializableHashMap;

#[derive(Clone, Debug)]
pub struct MCCFR<A: Action, S: State<A>> {
    game: Game<A, S>,
    pub iterations: usize,
    pub nodes_traversed: usize,
    strategies: Vec<RegretStrategy<A>>,
    game_mapper: GameMapper<A>,
    bonus: f64,
    exploration: f64,
    threshold: f64,
}

/// [Neal] Represents the state information necessary to run iterations on MCCFR
/// given a particular Game description
///
/// Original author used Outcome (or Chance) Sampling, we have switched this
/// to Average Sampling which is in theory a bit more efficient.
///
/// Tabular representation, which may be problematic for extra large games,
/// we may have to switch to a more efficient representation of the strategy or
/// DQNs in the future if we want to scale to larger abstract representations
///
/// There were also interesting ideas of using bincode to squeeze and compress the strategy
/// a very helpful article can be found here on the sorts of compressions you can do:
/// https://blog.logrocket.com/rust-serialization-whats-ready-for-production-today/
impl<A: Action, S: State<A>> MCCFR<A, S> {
    pub fn new(game: Game<A, S>) -> Self {
        let mut s = Vec::new();
        s.resize(game.num_regular_players(), RegretStrategy::default());
        MCCFR {
            game,
            iterations: 0,
            nodes_traversed: 0,
            strategies: s,
            game_mapper: GameMapper::new(None),
            bonus: 0.00, // Set to 0.0 and threshold to 1.0 for MCCFR Outcome Sampling
            exploration: 0.6,
            threshold: 1.0,
        }
    }

    pub fn with_game_mapper(&mut self, game_mapper: GameMapper<A>) {
        self.game_mapper = game_mapper;
    }

    pub fn write_to(&self, file_name: &str) {
        for i in 0..self.game.num_regular_players() {
            let file = format!("{}{}", file_name.to_owned(), format!("_p{}.json", i));
            self.strategies[i].save_table_json(&file, &self.game_mapper);
        }
    }

    pub fn run_averaging_iteration<R: Rng>(
        &mut self,
        rng: &mut R,
        updated_player: usize,
        depth: usize,
        q: f64, // Probability for bookkeeping a la AS MCCFR paper
    ) -> f64 {
        self.nodes_traversed += 1;
        match self.game.active_player() {
            ActivePlayer::Terminal(utilities) => {
                utilities[updated_player] / q
            }
            ActivePlayer::Chance(actions) => {
                let (action, _) = actions.sample_and_prob(rng);
                let mut action = self.game_mapper.map_action(action, depth);
                if actions.items().len() == 3 {
                    action = actions.items()[2].clone();
                }
                self.game.play(action);
                self.run_averaging_iteration(rng, updated_player, depth + 1, q)
            }
            ActivePlayer::Player(player_num, actions) => {
                let actions = self.game_mapper.map_actions(&actions, depth);
                let player_num = player_num as usize;
                let length = actions.len() as f64;
                let history = self.game.history(player_num);
                let strategy = &mut self.strategies[player_num];
                let mut regrets = match strategy.regrets(&history) {
                    Some(r) => regret_matching(r),
                    None => vec![1.0 / length; actions.len()],
                };

                if player_num != updated_player {
                    // Weigh actions by amount of regret accumulated
                    // for not taking the action
                    regrets = regrets.iter().map(|r| r / q).collect();
                    strategy.update(history, None, Some(&regrets));
                    let distribution = Categorical::new_normalized(regrets, actions);
                    let (sampled_action, probability) = distribution.sample_and_prob(rng);

                    // Sample and explore action (likelier to be one with higher regret)
                    self.game.play(sampled_action);
                    return self.run_averaging_iteration(rng, updated_player, depth + 1, q);
                }

                // Sample the policy (strategy that we've been learning)
                if strategy.policy(&history).is_none() {
                    let zeroes = vec![0.0; actions.len()];
                    strategy.update(history.clone(), None, Some(&zeroes));
                }
                let policy = strategy.policy(&history).expect("Could not get policy");
                let sampling_values =
                    average_sampling(&policy, self.exploration, self.bonus, self.threshold);

                let mut regret_updates: Vec<f64> = vec![];

                // Sample potentially many actions, and determine a
                // counterfactual regret update for each
                for (index, probability) in sampling_values.iter().enumerate() {
                    let will_sample = rng.gen_range(0.0, 1.0);
                    if will_sample < *probability {
                        // TODO: undo rather than clone
                        let temp_game = self.game.clone();
                        self.game.play(actions[index].clone());
                        let value = self.run_averaging_iteration(
                            rng,
                            updated_player,
                            depth + 1,
                            q * probability.min(1.0),
                        );
                        self.game = temp_game;
                        regret_updates.push(value);
                    } else {
                        regret_updates.push(0.0);
                    }
                }

                // Estimate the true value of each action using the above samples
                let counter_factual_estimation = regret_updates
                    .iter()
                    .zip(regrets.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f64>();

                // black magic ???
                let full_update = regret_updates
                    .iter()
                    .map(|a| a - counter_factual_estimation)
                    .collect::<Vec<f64>>();

                let strategy = &mut self.strategies[player_num];
                strategy.update(history, Some(&full_update), None);

                counter_factual_estimation
            }
        }
    }

    /// The specific math associated with the MCCFR algorithm,
    /// change here if you'd like to update the algo
    /// returns (utility, p_tail, p_sample_leaf)
    fn run_iteration<R: Rng>(
        &mut self,
        rng: &mut R,
        updated_player: usize,
        p_reach_updated: f64,
        p_reach_others: f64,
        p_sample: f64,
        epsilon: f64,
        depth: usize,
    ) -> (f64, f64, f64) {
        self.nodes_traversed += 1;
        match self.game.active_player() {
            ActivePlayer::Terminal(ref payoffs) => (payoffs[updated_player], 1.0, p_sample),
            ActivePlayer::Chance(ref cat) => {
                // Sample an action from the space of random chance
                // then map it to our internal action space
                let action = cat.sample_ref_rng(rng).clone();
                let action = self.game_mapper.map_action(action, depth);
                self.game.play(action);

                self.run_iteration(
                    rng,
                    updated_player,
                    p_reach_updated,
                    p_reach_others,
                    p_sample,
                    epsilon,
                    depth + 1,
                )
            }
            ActivePlayer::Player(player, ref actions) => {
                let player = player as usize;
                let actions = self.game_mapper.map_actions(actions, depth);
                let n = actions.len();
                let epsilon = if player == updated_player {
                    epsilon
                } else {
                    0.0
                };
                let history = self.game.history(player);
                let regret: Option<_> = self.strategies[player].get(&history).map(|e| &e.1);
                // Use CFR+ regret matching instead of vanilla regret matching
                let dist = match regret {
                    Some(r) => regret_matching(r),
                    None => vec![1.0 / n as f64; n],
                };

                let a_sample = if rng.sample::<f64, _>(rand::distributions::Standard) < epsilon {
                    rng.gen_range(0, n)
                } else {
                    crate::distribution::sample_weighted(&dist, rng)
                };
                let p_dist = dist[a_sample];
                let p_eps = epsilon / (n as f64) + (1.0 - epsilon) * p_dist;

                let action = actions[a_sample].clone();
                self.game.play(action);
                if player == updated_player {
                    let (payoff, p_tail, p_sample_leaf) = self.run_iteration(
                        rng,
                        updated_player,
                        p_reach_updated * p_dist,
                        p_reach_others,
                        p_sample * p_eps,
                        epsilon,
                        depth + 1,
                    );

                    let mut delta_regret = vec![0.0; n];
                    let u = payoff * p_reach_others / p_sample_leaf;
                    for action_index in 0..n {
                        if action_index == a_sample {
                            delta_regret[action_index] = u * (p_tail - p_tail * p_dist);
                        } else {
                            delta_regret[action_index] = -u * p_tail * p_dist;
                        }
                    }

                    self.strategies[player].update(history.clone(), Some(&delta_regret), None);
                    (payoff, p_tail * p_dist, p_sample_leaf)
                } else {
                    let (payoff, p_tail, p_sample_leaf) = self.run_iteration(
                        rng,
                        updated_player,
                        p_reach_updated,
                        p_reach_others * p_dist,
                        p_sample * p_eps,
                        epsilon,
                        depth + 1,
                    );
                    let mut distribution = dist;
                    distribution.iter_mut().for_each(|v| {
                        *v *= p_reach_updated / p_sample_leaf;
                    });
                    self.strategies[player].update(history.clone(), None, Some(&distribution));
                    (payoff, p_tail * p_dist, p_sample_leaf)
                }
            }
        }
    }
}

/// Average sampling used in line with this paper:
/// https://proceedings.neurips.cc/paper_files/paper/2012/file/3df1d4b96d8976ff5986393e8767f5b2-Paper.pdf
fn average_sampling(policy: &[f64], e: f64, b: f64, t: f64) -> Vec<f64> {
    let denominator: f64 = policy.iter().sum::<f64>() + b;
    let probabilities = policy
        .iter()
        .map(|s| (b + t * s) / denominator)
        .map(|s| s.max(e))
        .collect::<Vec<f64>>();
    probabilities
}

/// Weigh regrets by the relative size of that regret
fn regret_matching(reg: &[f64]) -> Vec<f64> {
    let regp = reg.iter().map(|&v| if v >= 0.0 { v } else { 0.0 });
    let s = regp.clone().sum::<f64>();
    let l = reg.len();

    // space optimization: caps the regret to not go infinitely negative
    // which means we can compress far more efficiently (reduced entropy)
    if s > 0.0 {
        regp.map(|v| v / s).collect()
    } else {
        vec![1.0 / l as f64; l]
    }
}
