use std::fmt::Debug;
use std::hash::Hash;

use crate::action::Action;
use crate::{ActionIndex, ActivePlayer, HistoryInfo, Observation};

pub trait Game: Debug + Clone {
    type State: Clone + Debug;
    type Observation: Action;
    type Action: Action + Into<ActionIndex>;

    fn players(&self) -> usize;

    fn initial_state(&self) -> (Self::State, ActivePlayer<Self>);

    /// [Neal] Describes how a game state is updated by a given action
    /// must compute what information is privately/publicly observed by each player
    /// and update the active player as well as the invariants in the state
    fn update_state(
        &self,
        hist: &HistoryInfo<Self>,
        action: &Self::Action,
    ) -> (
        Self::State,
        ActivePlayer<Self>,
        Vec<Option<Self::Observation>>,
    );

    fn start(&self) -> HistoryInfo<Self> {
        let (state, active) = self.initial_state();
        HistoryInfo::new(self, state, active)
    }

    /// [Neal] Helper method for play_owned
    ///
    /// Convert an action into an ActionIndex and then call play_owned
    fn play_value(&self, hist: &HistoryInfo<Self>, action: &Self::Action) -> HistoryInfo<Self> {
        if let ActivePlayer::Terminal(_) = hist.active {
            panic!("playing in terminal state {:?}", hist);
        }
        match hist.active.actions().iter().position(|x| x == action) {
            Some(idx) => self.play(&hist, idx),
            None => panic!("action {:?} not available in state {:?}", action, hist),
        }
    }

    /// [Neal] Helper method for play_owned (copies the reference presumably because the
    /// author was fighting the borrow checker)
    ///
    /// Simplified entry point for advancing the game by a given ActionIndex
    fn play(&self, hist: &HistoryInfo<Self>, action_index: usize) -> HistoryInfo<Self> {
        self.play_owned(hist.clone(), action_index)
    }

    /// Advance the game by a single ActionIndex, update the history to new valid state
    fn play_owned(&self, hist: HistoryInfo<Self>, action_index: usize) -> HistoryInfo<Self> {
        // Initial checks
        debug_assert_eq!(hist.observations.len(), self.players() + 1);
        if let ActivePlayer::Terminal(_) = hist.active {
            panic!("playing in terminal state {:?}", hist);
        }
        // Extract action
        let action = hist
            .active
            .actions()
            .get(action_index as usize)
            .expect("action index outside action list")
            .clone();
        // Game-specific logic and checks
        let (state, active, obs) = self.update_state(&hist, &action);
        debug_assert_eq!(obs.len(), self.players() + 1);
        if let ActivePlayer::Player(p, _) = active {
            debug_assert!((p as usize) < self.players());
        }
        // Dismantle hist
        let mut history_indices = hist.history_indices;
        let mut history = hist.history;
        let mut observations = hist.observations;
        // Observation extensions by own action
        if let Some(p) = hist.active.player() {
            observations[p].push(Observation::Private(action.clone()));
        }
        history_indices.push(action_index as ActionIndex);
        history.push(action);
        // Observation extension by new observed
        for (ovec, option_ob) in observations.iter_mut().zip(obs) {
            if let Some(ob) = option_ob {
                ovec.push(Observation::Public(ob))
            }
        }
        HistoryInfo {
            state,
            active,
            history,
            history_indices,
            observations,
        }
    }
}
