use crate::action::{HotEncoding, IntoHotEncoding};
use crate::{ActionIndex, Categorical, Game, Utility};
use std::{fmt::Debug, hash::Hash};

/// [Neal] Represents the actions that are made by a player
/// or not made directly by a player but still observed by them
///
/// This is perhaps to deal with information sets where the player cannot see all
/// actions that are made by other players, but can still observe some of them
///
/// For example, in poker, the random "chance" player takes the DealCards action with limited
/// observability to the other players, but the other players can still observe
/// the cards that are dealt to themselves
///
/// There is some design friction I can't quite place -
/// Perhaps this is better named as Private and Public observations, or actions
/// are better classified as Private and Public actions????
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Observation<A, O>
where
    A: Clone + Hash + Debug + PartialEq + Eq + Into<ActionIndex>,
    O: Clone + Hash + Debug + PartialEq + Eq + IntoHotEncoding,
{
    Private(A), // Stores actions that are made by the associated player
    Public(O),  // Stores actions that are observed by some player but not made by them TODO: (?)
}

impl<A, O> IntoHotEncoding for Observation<A, O>
where
    A: Clone + Hash + Debug + PartialEq + Eq + Into<ActionIndex>,
    O: Clone + Hash + Debug + PartialEq + Eq + IntoHotEncoding,
{
    fn encoding(self) -> HotEncoding {
        match self {
            Observation::Private(a) => {
                let a = a.into();
                a.encoding()
            }
            Observation::Public(o) => o.encoding(),
        }
    }
}

/// [Neal] Defines the unique type that will be used for information sets
/// in the game, based on the observability of each given action
///
/// Note: information sets are equal only if they contain the same public actions,
/// for example, in poker, players are not able to distinguish between an opponent
/// privately having an Ace or a King,
/// and can only observe public actions -> bets, folds, raises, and calls.
#[allow(type_alias_bounds)]
pub type PlayerObservation<G: Game> = Observation<G::Action, G::Observation>;

/// [Neal] Defines a player in the game currently about to take a turn
#[derive(Clone, Debug, PartialEq)]
pub enum ActivePlayer<G: Game> {
    Player(u32, Vec<G::Action>), // Label (P1, P2) and the possible actions they can take
    Chance(Categorical<G::Action>), // A random event that can occur (for example, a card deal)
    Terminal(Vec<Utility>), // The game has ended, with an internal state representing the value
                            // at this node. Do not be confused, this Vec<Utility> is used to
                            // calculate a single utility value for each player, and does not
                            // represent a choice of utilities
}

impl<G: Game> ActivePlayer<G> {
    #[inline]
    pub fn actions<'a>(&'a self) -> &'a [G::Action] {
        match self {
            ActivePlayer::Terminal(_) => &[],
            ActivePlayer::Player(_, ref actions) => actions,
            ActivePlayer::Chance(ref dist) => dist.items(),
        }
    }

    #[inline]
    pub fn player<'a>(&'a self) -> Option<usize> {
        match self {
            ActivePlayer::Terminal(_) => None,
            ActivePlayer::Player(p, _) => Some(*p as usize),
            ActivePlayer::Chance(_) => None,
        }
    }
}

/// [Neal] Seems to be a struct representing the history
/// that leads up to a certain point in the game, storing every action
/// (by either a player or by a  random event, labeled as "chance") under
/// HistoryInfo::history
///
/// HistoryInfo::history_indices stores the actions of each history as indices
/// this is due to there being a deterministic way to enumerate the actions, and so a
/// given Action at some point in the history is always recalculabe
///
#[derive(Clone, Debug)]
pub struct HistoryInfo<G: Game> {
    pub history_indices: Vec<ActionIndex>,
    pub history: Vec<G::Action>,
    pub active: ActivePlayer<G>,
    pub observations: Vec<Vec<PlayerObservation<G>>>, // Represents all information sets??
    // or information sets per player??
    // TODO: I think this is correct but
    // verify
    pub state: G::State, // All state information about the particular game
}

impl<G: Game> HistoryInfo<G> {
    pub fn new(game: &G, state: G::State, active: ActivePlayer<G>) -> Self {
        HistoryInfo {
            history_indices: Vec::new(),
            history: Vec::new(),
            observations: vec![vec! {}; (game.players() + 1) as usize],
            state,
            active,
        }
    }

    pub fn observations_since<'a>(&'a self, other: &Self) -> Vec<&'a [PlayerObservation<G>]> {
        self.observations
            .iter()
            .zip(other.observations.iter())
            .map(|(so, oo)| &so[oo.len()..])
            .collect()
    }
}
