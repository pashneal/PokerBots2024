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
pub struct ObservationTracker {
    player_info_sets: Vec<Vec<ActionIndex>>,
}

/// Observable features of a typical poker game 
/// that can be used to convey information
///
/// Thoughts about the EV feature:
///     - EV has the crucial flaw of assuming that the
///       opponent's hands are uniformly random, which is
///       only true of shitty players
///     - it may or may not be a good abstraction
///     - something that's going for it is that we can also include 
///     - information gleaned from the opponent's actions
///     - it's also a good abstraction because it's a single number  
///       (very simple to start off with)
///     - it does require us to have a blazingly fast evaluator hehehehehhehe
///       (which we don't yet but I'd much rather work on that instead of this)
pub enum Feature {
    Suited(bool),  // True if the hand is suited
    Ranks(u8, u8), // Sorted from highest to lowest
    EV(u8),        // Expected value of the hand as a percentage (0-100) 
}


pub enum WeightedEval {
    RoyalFlush,
    StraightFlush(u8),
    FourOfAKind(u8),
    FullHouse(u8, u8),
    Flush(u8),
    Straight(u8),
    ThreeOfAKind(u8),
    TwoPair(u8),
    Pair(u8),
    HighCard(u8),
}

pub enum Eval {
    RoyalFlush,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
}

pub enum Texture {

    Pair(usize),
    
}

pub enum Observation<A> {
    Action(A),
    Features(Vec<Feature>),
}

/// Represents the visibility of a given action to
/// all players within a game
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Visibility<A : Action> {
    Public(A),             //  All players can see the action
    Private(A),            // only a single player can see the action
    Shared(A, Vec<usize>), // A subset of players can see the action
}

impl ObservationTracker {
    pub fn new() -> Self {
        ObservationTracker {
            player_info_sets: vec![Vec::new(); NUM_REGULAR_PLAYERS],
        }
    }

    pub fn get_history(&self, player: usize) -> History {
        History(self.player_info_sets[player].clone())
    }

    pub fn observe<A: Action> (&mut self, visibility: Visibility<A>, active_player_index : Option<usize>) {
        match visibility {
            Visibility::Public(action) => {
                for player in 0..NUM_REGULAR_PLAYERS {
                    self.player_info_sets[player].push(action.clone().into());
                }
            }
            Visibility::Private(action) => {
                if let Some(player_index) = active_player_index {
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
