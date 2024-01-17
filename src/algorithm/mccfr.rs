use crate::constants::MAX_GAME_DEPTH;
use crate::game_logic::action::GameMapper;
use crate::game_logic::action::{Action, ActionIndex};
use crate::game_logic::state::{ActivePlayer, State};
use crate::game_logic::strategy::*;
use crate::game_logic::visibility::{History, Feature};
use crate::implementations::auction::Card;
use crate::{Categorical, Game};
use rand::Rng;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MCCFR<A: Action, S: State<A>> {
    game: Game<A, S>,
    pub iterations: usize,
    pub nodes_traversed: usize,
    strategies: Vec<Arc<RegretStrategy>>,
    game_mapper: GameMapper<A>,
    bonus: f32,
    exploration: f32,
    threshold: f32,
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
    pub fn new(game: Game<A, S>, strategies: Vec<Arc<RegretStrategy>>) -> Self {
        MCCFR {
            game,
            iterations: 0,
            nodes_traversed: 0,
            strategies: strategies,
            game_mapper: GameMapper::new(None),
            bonus: 100.0, // bonus to exploration, Set to 0.0 and threshold to 1.0 for MCCFR Outcome Sampling
            exploration: 0.6,
            threshold: 10000.0,
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
    pub fn run_iterations<R: Rng>(&mut self, iterations: usize, epsilon: f32, rng: &mut R) {
        self.exploration = epsilon;
        for i in 0..iterations {
            for player in 0..self.game.num_regular_players() {
                self.game = Game::<_, _>::new();
                self.run_averaging_iteration(rng, player, 0, 1.0);
            }
            self.iterations += 1;
            if i % 1 == 0 {
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
        q: f32, // Probability for bookkeeping a la AS MCCFR paper
    ) -> f32 {

        match self.game.active_player() {
            ActivePlayer::Terminal(utilities) => {
                self.nodes_traversed += 1;
                if self.nodes_traversed % 100000 == 0 {
                    println!("Iteration: {}, Nodes Traversed: {}", self.iterations, self.nodes_traversed);
                }
                utilities[updated_player] / q
            }
            ActivePlayer::Chance(actions) => {
                self.nodes_traversed += 1;
                let (action, default_index) = actions.sample_and_index(rng);
                let default_index = default_index as ActionIndex;
                let (action, index) = self.game_mapper.map_and_index(action, depth, default_index);
                self.game.play(&action);
                self.run_averaging_iteration(rng, updated_player, depth + 1, q)
            }
            ActivePlayer::Marker(action) => {
                self.game.play(&action);
                self.run_averaging_iteration(rng, updated_player, depth + 1, q)
            }

            ActivePlayer::Player(player_num, actions) => {
                self.nodes_traversed += 1;
                if self.nodes_traversed % 100000 == 0 {
                    println!("Iteration: {}, Nodes Traversed: {}", self.iterations, self.nodes_traversed);
                }
                let actions = self.game_mapper.map_actions(&actions, depth);
                let max_index = A::max_index();

                let mut mask = (0..max_index).map(|_| false).collect::<Vec<bool>>();
                let mut mapped_actions = (0..max_index)
                    .map(|_| None)
                    .collect::<Vec<Option<A>>>(); 

                for action in &actions {
                    mask[action.index() as usize] = true;
                    mapped_actions[action.index() as usize] = Some(action.clone());
                }

                let player_num = player_num as usize;
                let length = mask.len() as f32;

                let history = self.game.get_information_set(player_num);
                let strategy = &mut self.strategies[player_num];

                let mut regrets = match strategy.regrets(&history) {
                    Some(r) => regret_matching(&r,&mask),
                    None => vec![1.0 / length; length as usize],
                };

                if player_num != updated_player {
                    // Weigh actions by amount of regret accumulated
                    // for not taking the action
                    regrets = regrets.iter().map(|r| r / q).collect();
                    strategy.update(history, None, Some(&regrets));

                    // Discard actions that aren't legal and renormalize
                    let distribution = Categorical::new_normalized(regrets, mapped_actions);
                    debug_assert!(mask.iter().any(|a| *a));
                    // TODO: it is possible that the mask removed all positive regrets
                    // in which case we should just sample uniformly from the legal actions
                    let distribution = distribution.with_mask(&mask);
                    let (sampled_action, index) = distribution.sample_and_index(rng);


                    // Sample and explore action (likelier to be one with higher regret)
                    self.game.play(&sampled_action.unwrap());
                    return self.run_averaging_iteration(rng, updated_player, depth + 1, q);
                }

                // Sample the policy (strategy that we've been learning)
                if strategy.policy(&history).is_none() {
                    let zeroes = vec![0.0; length as usize];
                    strategy.update(history.clone(), None, Some(&zeroes));
                }
                let policy = strategy.policy(&history).expect("Could not get policy");

                let sampling_values =
                    average_sampling(&policy, self.exploration, self.bonus, self.threshold);

                let mut regret_updates: Vec<f32> = vec![];

                // Sample potentially many actions, and determine a
                // counterfactual regret update for each
                for (index, probability) in sampling_values.iter().enumerate() {
                    if !mask[index] {
                        regret_updates.push(0.0);
                        continue;
                    }

                    let will_sample = rng.gen_range(0.0, 1.0);
                    if will_sample < *probability {
                        // TODO: undo rather than clone
                        let temp_game = self.game.clone();
                        let selected_action = mapped_actions[index].as_ref().unwrap();
                        self.game.play(selected_action);
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

                // Estimate the true value of each action using the sum of above samples
                let counter_factual_estimation = regret_updates
                    .iter()
                    .zip(regrets.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f32>();

                let update_with_cfr = regret_updates
                    .iter()
                    .map(|a| a - counter_factual_estimation)
                    .collect::<Vec<f32>>();

                let dropped_non_actions = update_with_cfr
                    .iter()
                    .zip(mask.iter())
                    .map(|(a, b)| if *b { *a } else { 0.0 })
                    .collect::<Vec<f32>>();

                let strategy = &mut self.strategies[player_num];
                strategy.update(history, Some(&dropped_non_actions), None);
                //if self.nodes_traversed % 10000 < 1000 && player_num ==0 {
                    //let history : History = history.into();
                    //if history.0.len() == 5  {
                        //let cards = history.0[1].clone() as ActionIndex;
                        //let feature : Feature = cards.into();
                        //let policy = strategy.policy(&self.game.history(0)).unwrap();
                        //let new_regrets = strategy.regrets(&self.game.history(0)).unwrap();


                        //println!("history: {:?}", history);
                        //println!("feature: {:?}", feature);
                        //println!("suited: {:?}", history.0[2] == 1);
                        //println!("policy: {:?}", policy);
                        //println!("actions: {:?}", actions);
                        //println!("new reg: {:?}", new_regrets);
                        //println!("q: {}", q );
                        //println!("Value: {}", counter_factual_estimation);
                        //println!("Regrets updates: {:?}", dropped_non_actions);
                    //}

                //}

                counter_factual_estimation
            }
        }
    }
}

/// Average sampling used in line with this paper:
/// https://proceedings.neurips.cc/paper_files/paper/2012/file/3df1d4b96d8976ff5986393e8767f5b2-Paper.pdf
fn average_sampling(policy: &[f32], e: f32, b: f32, t: f32) -> Vec<f32> {
    let denominator: f32 = policy.iter().sum::<f32>() + b;
    let probabilities = policy
        .iter()
        .map(|s| (b + t * s) / denominator)
        .map(|s| s.max(e))
        .collect::<Vec<f32>>();
    probabilities
}

/// Weigh regrets by the relative size of that regret
fn regret_matching(reg: &[f32], mask : &[bool]) -> Vec<f32> {
    let regp = reg.iter().map(|&v| if v >= 0.0 { v } else { 0.0 });
    let regp = regp.zip(mask.iter()).map(|(r, m)| if *m { r } else { 0.0 });

    let s = regp.clone().sum::<f32>();
    let l = reg.len();

    // space optimization: caps the regret to not go infinitely negative
    // which means we can compress far more efficiently (reduced entropy)
    if s > 0.0 {
        regp.map(|v| v / s).collect()
    } else {
        vec![1.0 / l as f32; l]
    }
}
