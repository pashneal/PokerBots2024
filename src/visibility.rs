use crate::action::Action;
use crate::constants::*;
use crate::state::ActivePlayer;
use crate::strategy::InformationSet;
use std::{fmt::Debug, hash::Hash};

pub type History<A> = Vec<A>;
#[derive(Clone, Debug)]
pub struct VisibilityTracker<A: Action> {
    player_info_sets: Vec<InformationSet<A>>,
}

/// Represents the visibility of a given action to
/// all players within a game
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum Visibility<A: Action> {
    Public(A),             //  All players can see the action
    Private(A),            // Only a single player can see the action
    Shared(A, Vec<usize>), // A subset of players can see the action
}

impl<A: Action> VisibilityTracker<A> {
    pub fn new() -> Self {
        VisibilityTracker {
            player_info_sets: vec![Vec::new(); NUM_REGULAR_PLAYERS],
        }
    }

    pub fn get_history(&self, player: usize) -> InformationSet<A> {
        self.player_info_sets[player].clone()
    }

    pub fn observe(&mut self, visibility: Visibility<A>, active_player: &ActivePlayer<A>) {
        match visibility {
            Visibility::Public(action) => {
                for player in 0..NUM_REGULAR_PLAYERS {
                    self.player_info_sets[player].push(action.clone());
                }
            }
            Visibility::Private(action) => {
                if let Some(index) = active_player.as_index() {
                    self.player_info_sets[index].push(action);
                }
            }
            Visibility::Shared(action, players) => {
                for player in players {
                    self.player_info_sets[player].push(action.clone());
                }
            }
        }
    }
}
