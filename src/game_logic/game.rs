use crate::constants::*;
use crate::game_logic::state::State;
use crate::game_logic::strategy::CondensedInfoSet;
use crate::game_logic::visibility::ObservationTracker;
use std::fmt::Debug;
use std::hash::Hash;

use crate::game_logic::action::{Action, ActionIndex};
use crate::game_logic::state::ActivePlayer;

#[derive(Clone, Debug)]
pub struct Game<A: Action, S: State<A>>
where
    S: Clone,
{
    observation_tracker: ObservationTracker,
    state: S,
    action: std::marker::PhantomData<A>,
}

impl<A: Action, S: State<A>> Game<A, S>
where
    S: Clone,
{
    pub fn num_regular_players(&self) -> usize {
        NUM_REGULAR_PLAYERS
    }

    pub fn new() -> Self {
        Game {
            state: S::new(),
            observation_tracker: ObservationTracker::new(),
            action: std::marker::PhantomData,
        }
    }

    /// Advance the game by a single Action
    pub fn play(&mut self, action: A) {
        let active_player = self.state.active_player();
        let observations = self.state.get_observations(&action);
        self.observation_tracker
            .observe_all(observations, active_player.as_index());
        self.state.update(action);
    }

    pub fn history(&self, player: usize) -> CondensedInfoSet {
        self.observation_tracker.get_history(player).into_condensed()
    }

    pub fn active_player(&self) -> ActivePlayer<A> {
        self.state.active_player()
    }
}
