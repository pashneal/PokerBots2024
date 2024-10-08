use crate::constants::*;
use crate::game_logic::action::{Action, ActionIndex};
use crate::game_logic::state::ActivePlayer;
use crate::game_logic::strategy::CondensedInfoSet;
use std::{fmt::Debug, hash::Hash};
use crate::implementations::auction::Card;

#[derive(Clone, Debug)]
pub struct History(pub Vec<ActionIndex>);

pub static MAX_ACTIONS: CondensedInfoSet = 200;
impl History {
    pub fn into_condensed(self) -> CondensedInfoSet {
        let mut condensed = 1;
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
        //while condensed > 0 {
        while condensed > 1 {
            history.push((condensed % MAX_ACTIONS) as ActionIndex);
            condensed /= MAX_ACTIONS;
        }
        History(history)
    }
}

#[derive(Clone, Debug)]
pub struct ObservationTracker {
    player_info_sets: Vec<Vec<ActionIndex>>,
    player_feature_sets: Vec<Option<Vec<Feature>>>,
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

#[derive(Clone, Debug)]
pub enum Round {
    PreFlop,
    Auction,
    Flop,
    Turn,
    River,
}

impl Into<usize> for Round {
    fn into(self) -> usize {
        match self {
            Round::PreFlop => 0,
            Round::Auction => 1,
            Round::Flop => 2,
            Round::Turn => 3,
            Round::River => 4,
        }
    }
}
impl From<usize> for Round {
    fn from(u : usize) -> Round {
        match u {
            0 => Round::PreFlop,
            1 => Round::Auction,
            2 => Round::Flop,
            3 => Round::Turn,
            4 => Round::River,
            _ => panic!("Cannot convert {} to Round", u),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BidResult {
    Player(u8),
    Tie,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Suited(bool),        // True if the hand is suited
    Ranks(usize, usize), // Sorted from highest to lowest
    EV(u16),             // Expected value of the hand as a percentage (0-100)
    Pot(u8),             // Pot size as a percentage of a stack (0-200)
    Order(Round),
    Auction(BidResult),
    Stack(u8), // Stack as percentage of max scaled down (0-50)
    Aggression(usize),
}


impl Feature {
    pub fn max_index() -> usize {
        200
    }
}


impl Into<ActionIndex> for Feature {
    fn into(self) -> ActionIndex {
        match self {
            Feature::Suited(x) => x as ActionIndex,
            Feature::Ranks(x, y) => x as ActionIndex * 13 + y as ActionIndex,
            Feature::EV(x) => x as ActionIndex,
            Feature::Pot(x) => x as ActionIndex,
            Feature::Order(round) => {
                let round_index: usize = round.into();
                round_index as ActionIndex
            }
            Feature::Auction(result) => match result {
                BidResult::Player(player) => player as ActionIndex,
                BidResult::Tie => 2,
            },
            Feature::Stack(x) => x as ActionIndex,
            Feature::Aggression(x) => x as ActionIndex,
        }
    }
}

impl From<ActionIndex> for Feature {
    fn from(index :ActionIndex) -> Self{
       match index  {
           0..=169 => {
               let rank1 = (index / 13) as usize;
               let rank2 = (index % 13) as usize;
               Feature::Ranks(rank1, rank2)
           }
           _ => panic!("Invalid feature index")
       }

    }
}

#[derive(Clone, Debug)]
pub enum Information<A> {
    Action(A),
    Features(Vec<Feature>),
    Discard,
}

/// Represents the visibility of a given action to
/// all players within a game
#[derive(Clone, Debug)]
pub enum Observation<A: Action> {
    Public(Information<A>),             //  All players can see the action
    Private(Information<A>),            // only a single player can see the action
    Shared(Information<A>, Vec<usize>), // A subset of players can see the action
}

impl ObservationTracker {
    pub fn new() -> Self {
        ObservationTracker {
            player_info_sets: vec![Vec::new(); NUM_REGULAR_PLAYERS],
            player_feature_sets: vec![None; NUM_REGULAR_PLAYERS],
        }
    }

    pub fn get_history(&self, player: usize) -> History {
        if let Some(history) = &self.player_feature_sets[player] {
            let action_indices = history.iter().map(|action| action.clone().into()).collect();
            History(action_indices)
        } else {
            History(self.player_info_sets[player].clone())
        }
    }

    pub fn observe_all<A: Action>(
        &mut self,
        observations: Vec<Observation<A>>,
        active_player_index: Option<usize>,
    ) {
        for observation in observations {
            self.observe(observation, active_player_index);
        }
    }

    pub fn observe<A: Action>(
        &mut self,
        observation: Observation<A>,
        active_player_index: Option<usize>,
    ) {
        match observation {
            Observation::Public(info) => match info {
                Information::Action(action) => {
                    for player in 0..NUM_REGULAR_PLAYERS {
                        self.player_info_sets[player].push(action.clone().into());
                    }
                }
                Information::Features(features) => {
                    for player in 0..NUM_REGULAR_PLAYERS {
                        self.player_feature_sets[player] = Some(features.clone());
                    }
                }
                _ => {}
            },

            Observation::Private(info) => match info {
                Information::Action(action) => {
                    if let Some(player_index) = active_player_index {
                        self.player_info_sets[player_index].push(action.clone().into());
                    }
                }
                Information::Features(features) => {
                    if let Some(player_index) = active_player_index {
                        self.player_feature_sets[player_index] = Some(features.clone());
                    }
                }
                _ => {}
            },

            Observation::Shared(info, players) => match info {
                Information::Action(action) => {
                    for player in players {
                        self.player_info_sets[player].push(action.clone().into());
                    }
                }
                Information::Features(features) => {
                    for player in players {
                        self.player_feature_sets[player] = Some(features.clone());
                    }
                }
                _ => {}
            },
        }
    }
}
