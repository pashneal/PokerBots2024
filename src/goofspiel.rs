/// Implementation of Goofspiel, a simpler card game. Very useful for
/// figuring out how to implement a game in this framework.
use crate::action::{HotEncoding, IntoHotEncoding};
use crate::constants::HOT_ENCODING_SIZE;
use crate::state::{ActivePlayer, State};
use crate::visibility::Visibility;
use crate::{Categorical, Utility};
use bit_set::BitSet;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Scoring {
    ZeroSum,
    WinLoss,
    Absolute,
}

const MIN_SCORE: i32 = -13;

impl IntoHotEncoding for i32 {
    fn encoding(self) -> HotEncoding {
        let score = self - MIN_SCORE;
        let mut v = vec![false; HOT_ENCODING_SIZE];
        v[score as usize] = true;
        v
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Goofspiel {
    /// Number of cards.
    pub cards: usize,
    /// Final scoring type.
    pub scoring: Scoring,
    /// Point values of the chance cards (cards in hands always have strength 0..N-1).
    pub values: Vec<Utility>,
    /// Cached full-hand bit set
    card_set: BitSet,
}

impl Goofspiel {
    pub fn new(cards: usize, scoring: Scoring) -> Self {
        Self::with_values(
            cards,
            scoring,
            (1..cards + 1).map(|x| x as Utility).collect::<Vec<_>>(),
        )
    }

    pub fn with_values<V: Into<Vec<Utility>>>(cards: usize, scoring: Scoring, values: V) -> Self {
        Goofspiel {
            cards,
            scoring,
            card_set: (1..cards + 1).collect(),
            values: values.into(),
        }
    }
}

pub type GoofspielAction = u32;

/// Players are p0 and p1, p2 is chance
#[derive(Clone, Debug)]
pub struct GoofspielState {
    cards: [BitSet; 3],
    scores: [f64; 2],
    active: ActivePlayer<GoofspielAction>,
    bets: [GoofspielAction; 2],
    internal: Goofspiel, // [Neal] This is poor design but it's
                         // because I don't really want to re-implement the above
                         // but just re-use the existing implementation
}

impl GoofspielState {
    fn terminal(&self) -> ActivePlayer<GoofspielAction> {
        let delta = self.scores[0] - self.scores[1];
        ActivePlayer::Terminal(match self.internal.scoring {
            Scoring::Absolute => self.scores.as_ref().into(),
            Scoring::ZeroSum => vec![delta, -delta],
            Scoring::WinLoss => vec![delta.signum(), -delta.signum()],
        })
    }

    fn player_update(&mut self, action: GoofspielAction) {
        if let ActivePlayer::Player(player_num, _) = self.active_player() {
            let player_num = player_num as usize;
            self.cards[player_num].remove(action as usize);
            self.bets[player_num] = action;
            let betting_round_over = player_num == 1;
            if betting_round_over {
                // If the betting round is over,
                // then we need to give the biggest better the points!
                let card_value = self.internal.values[(action - 1) as usize];
                let winner = (self.bets[0] as i32 - self.bets[1] as i32).signum();
                if winner == 1 {
                    self.scores[0] += card_value;
                }
                if winner == -1 {
                    self.scores[1] += card_value;
                }
                // Implicitly discard the card if it's a tie
            }

            let player1_cards = self.cards[1].iter().map(|x| x as GoofspielAction).collect();
            let player2_cards = self.cards[2]
                .iter()
                .map(|x| x as GoofspielAction)
                .collect::<Vec<_>>();
            let num_cards_remaining = player2_cards.len();

            let mut distribution = None;
            if num_cards_remaining > 0 {
                distribution = Some(Categorical::uniform(player2_cards));
            }

            // State machine logic determining the next player
            match player_num {
                0 => self.active = ActivePlayer::Player(1, player1_cards),
                1 => {
                    self.active = match num_cards_remaining {
                        1.. => ActivePlayer::Chance(distribution.unwrap()),
                        0 => self.terminal(),
                        _ => panic!("Invalid number of cards remaining"),
                    }
                }
                _ => panic!("Unsure how to handle player number {}", player_num),
            }
        } else {
            panic!("Player update called when active player is not a regular player")
        }
    }

    fn chance_update(&mut self, action: GoofspielAction) {
        // Choose a card and remove the chosen card from the chance pool
        self.cards[2].remove(action as usize);

        // Loop to player 0
        let available_cards = self.cards[0].iter().map(|x| x as GoofspielAction).collect();
        self.active = ActivePlayer::Player(0, available_cards);
    }
}

impl State<GoofspielAction> for GoofspielState {
    fn new() -> Self {
        let internal = Goofspiel::new(5, Scoring::ZeroSum);
        let cards = [
            internal.card_set.clone(),
            internal.card_set.clone(),
            internal.card_set.clone(),
        ];
        let scores = [0.0, 0.0];
        let active = ActivePlayer::Chance(Categorical::uniform(
            internal
                .card_set
                .iter()
                .map(|x| x as u32)
                .collect::<Vec<_>>(),
        ));
        let bets = [0, 0];
        GoofspielState {
            cards,
            scores,
            active,
            bets,
            internal,
        }
    }

    fn active_player(&self) -> ActivePlayer<GoofspielAction> {
        self.active.clone()
    }

    fn get_visibility(&self, action: &GoofspielAction) -> Visibility<GoofspielAction> {
        match self.active_player() {
            ActivePlayer::Terminal(_) => panic!("Terminal state has no visibility"),
            ActivePlayer::Player(_, _) => Visibility::Private(action.clone()),
            ActivePlayer::Chance(_) => Visibility::Public(action.clone()),
        }
    }

    fn update(&mut self, action: GoofspielAction) {
        match self.active_player() {
            ActivePlayer::Terminal(_) => panic!("Terminal state cannot be updated"),
            ActivePlayer::Player(_, _) => self.player_update(action),
            ActivePlayer::Chance(_) => self.chance_update(action),
        }
    }
}
