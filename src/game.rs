use crate::constants::*;
use crate::state::State;
use crate::strategy::InformationSet;
use crate::visibility::VisibilityTracker;
use std::fmt::Debug;
use std::hash::Hash;

use crate::action::Action;
use crate::state::ActivePlayer;

#[derive(Clone, Debug)]
pub struct Game<A: Action, S: State<A>>
where
    S: Clone,
{
    visibility_tracker: VisibilityTracker<A>,
    state: S,
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
            visibility_tracker: VisibilityTracker::new(),
        }
    }

    /// Advance the game by a single Action
    pub fn play(&mut self, action: A) {
        let active_player = self.state.active_player();
        let visibility = self.state.get_visibility(&action);
        self.visibility_tracker.observe(visibility, &active_player);
        self.state.update(action);
    }

    pub fn history(&self, player: usize) -> InformationSet<A> {
        self.visibility_tracker.get_history(player)
    }

    pub fn active_player(&self) -> ActivePlayer<A> {
        self.state.active_player()
    }
}
