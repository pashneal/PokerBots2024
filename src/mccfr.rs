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

    /// [Neal] Run the MCCFR iterations as specificed
    pub fn run_iterations<R: Rng>(&mut self, iterations: usize, epsilon: f64, rng: &mut R) {
        self.exploration = epsilon;
        for i in 0..iterations {
            for player in 0..self.game.num_regular_players() {
                self.strategies[player].iterations += 1;
                self.game = Game::<_, _>::new();
                self.run_averaging_iteration(rng, player, 0, 1.0);
                //self.run_iteration(rng, player, 1.0, 1.0, 1.0, epsilon, 0);
            }
            self.iterations += 1;
            if i % 100_000 == 0 {
                println!(
                    "Iteration: {}, Nodes Traversed: {}, strategies[0] size: {}",
                    self.iterations,
                    self.nodes_traversed,
                    self.strategies[0].size()
                );
            }
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
                    Some(r) => regret_matching(&r),
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
