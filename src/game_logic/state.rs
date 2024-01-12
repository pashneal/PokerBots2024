use crate::game_logic::action::Action;
use crate::game_logic::visibility::Observation;
use crate::{Categorical, Utility};

/// [Neal] Defines a player in the game currently about to take a turn
#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer<A: Action> {
    Player(u32, Vec<A>), // Label (P1, P2, etc) and the possible actions they can take
    Chance(Categorical<A>), // A random event that can occur (for example, a card deal)
    Terminal(Vec<Utility>), // The game has ended, with an internal state representing the value
                         // at this node. Do not be confused, this Vec<Utility> is used to
                         // calculate a single utility value for each player, and does not
                         // represent a choice of utilities
}

impl<A: Action> ActivePlayer<A> {
    #[inline]
    pub fn actions<'a>(&'a self) -> &'a [A] {
        match self {
            ActivePlayer::Terminal(_) => &[],
            ActivePlayer::Player(_, ref actions) => actions,
            ActivePlayer::Chance(ref dist) => dist.items(),
        }
    }

    #[inline]
    pub fn as_index(&self) -> Option<usize> {
        match self {
            ActivePlayer::Terminal(_) => None,
            ActivePlayer::Player(p, _) => Some(*p as usize),
            ActivePlayer::Chance(_) => None,
        }
    }

    pub fn player_num(&self) -> usize {
        match self {
            ActivePlayer::Terminal(_) => panic!("Terminal node has no player number"),
            ActivePlayer::Player(p, _) => *p as usize,
            ActivePlayer::Chance(_) => panic!("Chance node has no player number"),
        }
    }
}

pub trait State<A: Action>: Clone {
    /// Given a current state, determine a given action's visibility
    /// with respect to the active player.
    ///
    ///     Public: The action is visible to all players
    ///     Private: The action is only visible to the active player
    ///     Shared(Vec<_>): The action is visible to the players in the vector
    fn get_observations(&mut self, action: &A) -> Vec<Observation<A>>;
    //fn get_features<B : Into<ActionIndex> + Clone>(&self, action: &A) -> Vec<Visibility<B>>;
    /// Returns the current player in a given state
    fn active_player(&self) -> ActivePlayer<A>;
    /// Advance the state by a given action
    fn update(&mut self, action: A);
    /// Initialize a new state
    fn new() -> Self;
}
