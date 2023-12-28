use crate::action::{HotEncoding, IntoHotEncoding, GameMapper};
use crate::constants::MAX_GAME_DEPTH;
use crate::{
    ActionIndex, Categorical, Game
};
use crate::strategy::RegretStrategy;
use crate::action::Action;
use crate::state::{State, ActivePlayer};
use hashbrown::HashMap;
use rand::Rng;
use serde_json::json;
use std::collections::HashMap as SerializableHashMap;




#[derive(Clone, Debug)]
pub struct MCCFR<A: Action, S : State<A>> {
    pub game: Game<A, S>,
    pub iterations: usize,
    pub nodes_traversed: usize,
    pub strategies: Vec<RegretStrategy<A>>,
    pub action_mapper : GameMapper<A>,
}

/// [Neal] Represents the state information necessary to run iterations on MCCFR
/// given a particular Game description, the name OuterMCCFR might not be entirely accurate
/// (the author may have meant External Sampling MCCFR, but translated it to Outer)
///
/// Tabular representation, which may be problematic for extra large games,
/// we may have to switch to a more efficient representation of the strategy or
/// DQNs in the future if we want to scale to larger abstract representations
impl <A: Action, S : State<A>> MCCFR<A, S> {
    pub fn new(game: Game<A,S>) -> Self {
        let mut s = Vec::new();
        s.resize(game.num_regular_players(), RegretStrategy::default());
        MCCFR {
            game,
            iterations: 0,
            nodes_traversed: 0,
            strategies: s,
            action_mapper : GameMapper::new(None),
        }
    }

    pub fn write_to(&self, file_name: &str) {
        for i in 0..self.game.num_regular_players() {
            let file = format!("{}{}", file_name.to_owned(), format!("_p{}.json", i));
            self.strategies[i].save_table_json(&file);
        }
    }

    /// [Neal] Run the MCCFR iterations as specificed
    pub fn run_iterations<R: Rng>(&mut self, iterations: usize, epsilon: f64, rng: &mut R) {
        for _i in 0..iterations {
            for player in 0..self.game.num_regular_players() {
                self.strategies[player].iterations += 1;
                self.game = Game::<_,_>::new();
                self.run_iteration(rng, player, 1.0, 1.0, 1.0, epsilon, 0);
            }
            self.iterations += 1;
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
        depth : usize,
    ) -> (f64, f64, f64) {
        self.nodes_traversed += 1;
        match self.game.active_player() {
            ActivePlayer::Terminal(ref payoffs) => {
                (payoffs[updated_player], 1.0, p_sample)

            },
            ActivePlayer::Chance(ref cat) => {
                // Sample an action from the space of random chance
                // then map it to our internal action space
                let action = cat.sample_ref_rng(rng).clone();
                let action = self.action_mapper.map_action(action, depth);
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
                let actions = self.action_mapper.map_actions(actions, depth);
                let n = actions.len();
                let epsilon = if player == updated_player {
                    epsilon
                } else {
                    0.0
                };
                let history = self.game.history(player);
                let regret: Option<_> = self.strategies[player].get(&history).map(|e| &e.1);
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
                let history = self.game.history(player);
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

fn regret_matching(reg: &[f64]) -> Vec<f64> {
    let regp = reg.iter().map(|&v| if v >= 0.0 { v } else { 0.0 });
    let s = regp.clone().sum::<f64>();
    let l = reg.len();
    if s > 0.0 {
        regp.map(|v| v / s).collect()
    } else {
        vec![1.0 / l as f64; l]
    }
}

