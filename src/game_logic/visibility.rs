use crate::game_logic::action::{Action, ActionIndex};
use crate::constants::*;
use crate::game_logic::state::ActivePlayer;
use crate::game_logic::strategy::CondensedInfoSet;
use std::{fmt::Debug, hash::Hash};

#[derive(Clone, Debug)]
pub struct History(pub Vec<ActionIndex>);


// TODO:  deal with greater than just 8 actions
pub static MAX_ACTIONS : CondensedInfoSet = 8;
impl History {
    pub fn into_condensed(self) -> CondensedInfoSet {
        let mut condensed = 0;
        for action in self.0.iter().rev() {
            condensed *= MAX_ACTIONS;
            condensed += *action as CondensedInfoSet;
        }
        condensed
    }
}

impl From<CondensedInfoSet> for History {
    fn from(condensed: CondensedInfoSet) -> Self {
        let mut history = Vec::new();
        let mut condensed = condensed;
        while condensed > 0 {
            history.push((condensed % MAX_ACTIONS) as ActionIndex);
            condensed /= MAX_ACTIONS;
        }
        History(history)
    }
}


#[derive(Clone, Debug)]
pub struct VisibilityTracker {
    player_info_sets: Vec<Vec<ActionIndex>>,
}

/// Represents the visibility of a given action to
/// all players within a game
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Visibility<A: Action> {
    Public(A),             //  All players can see the action
    Private(A),            // Only a single player can see the action
    Shared(A, Vec<usize>), // A subset of players can see the action
}

impl VisibilityTracker {
    pub fn new() -> Self {
        VisibilityTracker {
            player_info_sets: vec![Vec::new(); NUM_REGULAR_PLAYERS],
        }
    }

    pub fn get_history(&self, player: usize) -> History {
        History(self.player_info_sets[player].clone())
    }

    pub fn observe<A: Action>(&mut self, visibility: Visibility<A>, active_player: &ActivePlayer<A>) {
        match visibility {
            Visibility::Public(action) => {
                for player in 0..NUM_REGULAR_PLAYERS {
                    self.player_info_sets[player].push(action.clone().into());
                }
            }
            Visibility::Private(action) => {
                if let Some(player_index) = active_player.as_index() {
                    self.player_info_sets[player_index].push(action.into());
                }
            }
            Visibility::Shared(action, players) => {
                for player in players {
                    self.player_info_sets[player].push(action.clone().into());
                }
            }
        }
    }
}
