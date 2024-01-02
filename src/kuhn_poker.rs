use crate::action::*;
use crate::distribution::Categorical;
use crate::state::{ActivePlayer, State};
use crate::visibility::Visibility;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum KuhnPokerAction {
    Fold,
    Call,
    Check,
    Deal(u8),
    Bet,
}

impl IntoHotEncoding for KuhnPokerAction {
    fn encoding(self, size: usize) -> HotEncoding {
        match self {
            KuhnPokerAction::Fold => vec![true, false, false, false],
            KuhnPokerAction::Call => vec![false, true, false, false],
            KuhnPokerAction::Check => vec![false, false, true, false],
            KuhnPokerAction::Deal(x) => {
                let mut v = vec![false; size];
                v[x as usize] = true;
                v
            }
            KuhnPokerAction::Bet => vec![false, false, false, true],
        }
    }
}

impl Into<u32> for KuhnPokerAction {
    fn into(self) -> u32 {
        match self {
            KuhnPokerAction::Fold => 0,
            KuhnPokerAction::Call => 1,
            KuhnPokerAction::Check => 2,
            KuhnPokerAction::Deal(x) => x as u32 + 3,
            KuhnPokerAction::Bet => 6,
        }
    }
}

impl Parsable for KuhnPokerAction {
    fn to_string(&self) -> Option<String> {
        None
    }

    fn to_usize(&self) -> Option<usize> {
        None
    }
}
impl Filterable for KuhnPokerAction {}
impl Action for KuhnPokerAction {}

#[derive(Debug, Clone)]
pub struct KuhnPokerState {
    cards: Vec<u32>,
    players_cards: [Option<u32>; 2],
    active_player: ActivePlayer<KuhnPokerAction>,
}

/// TODO: use test to determine that this converges to game theory optimal
impl KuhnPokerState {
    fn dealer(cards: Vec<u32>) -> ActivePlayer<KuhnPokerAction> {
        let mut deals = Vec::new();
        for card in cards {
            let c = card as u8;
            deals.push(KuhnPokerAction::Deal(c));
        }
        ActivePlayer::Chance(Categorical::uniform(deals))
    }

    fn folded(&self, delta: f32, player_num: usize) -> ActivePlayer<KuhnPokerAction> {
        if player_num == 1 {
            ActivePlayer::Terminal(vec![delta, -delta])
        } else {
            ActivePlayer::Terminal(vec![-delta, delta])
        }
    }

    fn showdown(&self, delta: f32) -> ActivePlayer<KuhnPokerAction> {
        let card0 = self.players_cards[0].unwrap();
        let card1 = self.players_cards[1].unwrap();

        if card0 > card1 {
            ActivePlayer::Terminal(vec![delta, -delta])
        } else {
            ActivePlayer::Terminal(vec![-delta, delta])
        }
    }
}

impl State<KuhnPokerAction> for KuhnPokerState {
    fn new() -> Self {
        let cards = vec![0, 1, 2];
        let active_player = KuhnPokerState::dealer(cards.clone());
        KuhnPokerState {
            cards,
            players_cards: [None, None],
            active_player,
        }
    }

    fn get_visibility(&self, action: &KuhnPokerAction) -> Visibility<KuhnPokerAction> {
        match action {
            KuhnPokerAction::Fold => Visibility::Public(KuhnPokerAction::Fold),
            KuhnPokerAction::Call => Visibility::Public(KuhnPokerAction::Call),
            KuhnPokerAction::Check => Visibility::Public(KuhnPokerAction::Check),
            KuhnPokerAction::Deal(x) => match self.players_cards {
                [None, None] => Visibility::Shared(KuhnPokerAction::Deal(*x), vec![0]),
                [Some(_), None] => Visibility::Shared(KuhnPokerAction::Deal(*x), vec![1]),
                _ => panic!("This is not a valid state!"),
            },
            KuhnPokerAction::Bet => Visibility::Public(KuhnPokerAction::Bet),
        }
    }

    fn active_player(&self) -> ActivePlayer<KuhnPokerAction> {
        return self.active_player.clone();
    }

    fn update(&mut self, action: KuhnPokerAction) {
        match action {
            KuhnPokerAction::Fold => {
                let player_num = self.active_player.player_num();
                self.active_player = self.folded(1.0, player_num);
            }
            KuhnPokerAction::Call => {
                self.active_player = self.showdown(2.0);
            }
            KuhnPokerAction::Check => {
                let player_num = self.active_player.player_num();
                if player_num == 0 {
                    let actions = vec![KuhnPokerAction::Check, KuhnPokerAction::Bet];
                    self.active_player = ActivePlayer::Player(1, actions);
                } else {
                    self.active_player = self.showdown(1.0);
                }
            }
            KuhnPokerAction::Deal(x) => {
                if self.players_cards[0] == None {
                    self.players_cards[0] = Some(x as u32);
                    self.cards.remove(x as usize);
                    self.active_player = KuhnPokerState::dealer(self.cards.clone());
                } else {
                    self.players_cards[1] = Some(x as u32);
                    let actions = vec![KuhnPokerAction::Check, KuhnPokerAction::Bet];
                    self.active_player = ActivePlayer::Player(0, actions);
                }
            }
            KuhnPokerAction::Bet => {
                let actions = vec![KuhnPokerAction::Fold, KuhnPokerAction::Call];
                let other_player = (self.active_player.player_num() ^ 1) as u32;
                self.active_player = ActivePlayer::Player(other_player, actions);
            }
        }
    }
}
