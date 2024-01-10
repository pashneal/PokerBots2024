use crate::constants::*;
use crate::distribution::Categorical;
use crate::eval::rank::HandRanker;
use crate::game_logic::action::*;
use crate::game_logic::state::{ActivePlayer, State};
use crate::game_logic::visibility::{Information, Observation};
use rand::prelude::*;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Parsable for Suit {
    fn to_string(&self) -> Option<String> {
        match self {
            Suit::Hearts => Some("h".to_string()),
            Suit::Diamonds => Some("d".to_string()),
            Suit::Clubs => Some("c".to_string()),
            Suit::Spades => Some("s".to_string()),
        }
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Suit {
    fn new(s: String) -> Self {
        match s.as_str() {
            "h" => Suit::Hearts,
            "d" => Suit::Diamonds,
            "c" => Suit::Clubs,
            "s" => Suit::Spades,
            _ => panic!("Invalid suit string"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Parsable for Value {
    fn to_string(&self) -> Option<String> {
        let result = match self {
            Value::Two => "2".to_string(),
            Value::Three => "3".to_string(),
            Value::Four => "4".to_string(),
            Value::Five => "5".to_string(),
            Value::Six => "6".to_string(),
            Value::Seven => "7".to_string(),
            Value::Eight => "8".to_string(),
            Value::Nine => "9".to_string(),
            Value::Ten => "T".to_string(),
            Value::Jack => "J".to_string(),
            Value::Queen => "Q".to_string(),
            Value::King => "K".to_string(),
            Value::Ace => "A".to_string(),
        };
        Some(result)
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Value {
    fn new(s: String) -> Self {
        match s.as_str() {
            "2" => Value::Two,
            "3" => Value::Three,
            "4" => Value::Four,
            "5" => Value::Five,
            "6" => Value::Six,
            "7" => Value::Seven,
            "8" => Value::Eight,
            "9" => Value::Nine,
            "T" => Value::Ten,
            "J" => Value::Jack,
            "Q" => Value::Queen,
            "K" => Value::King,
            "A" => Value::Ace,
            _ => panic!("Invalid value string"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
}

impl Parsable for Card {
    fn to_string(&self) -> Option<String> {
        let string = format!(
            "{}{}",
            self.value.to_string().unwrap(),
            self.suit.to_string().unwrap()
        );
        Some(string)
    }
    fn to_usize(&self) -> Option<usize> {
        let suit = match self.suit {
            Suit::Hearts => 0,
            Suit::Diamonds => 1,
            Suit::Clubs => 2,
            Suit::Spades => 3,
        };
        let value = match self.value {
            Value::Ace => 0,
            Value::King => 1,
            Value::Queen => 2,
            Value::Jack => 3,
            Value::Ten => 4,
            Value::Nine => 5,
            Value::Eight => 6,
            Value::Seven => 7,
            Value::Six => 8,
            Value::Five => 9,
            Value::Four => 10,
            Value::Three => 11,
            Value::Two => 12,
        };
        let index = suit + value * 4;
        Some(index)
    }
}
pub type CardIndex = usize;
impl Card {
    pub fn from_index(index: CardIndex) -> Self {
        let suit = match index % 4 {
            0 => Suit::Hearts,
            1 => Suit::Diamonds,
            2 => Suit::Clubs,
            3 => Suit::Spades,
            _ => panic!("Invalid value index"),
        };
        let value = match index / 4 {
            0 => Value::Ace,
            1 => Value::King,
            2 => Value::Queen,
            3 => Value::Jack,
            4 => Value::Ten,
            5 => Value::Nine,
            6 => Value::Eight,
            7 => Value::Seven,
            8 => Value::Six,
            9 => Value::Five,
            10 => Value::Four,
            11 => Value::Three,
            12 => Value::Two,
            _ => panic!("Invalid suit index"),
        };
        Card { value, suit }
    }
    pub fn new(s: &str) -> Self {
        let mut chars = s.chars();
        let value = Value::new(chars.next().unwrap().to_string());
        let suit = Suit::new(chars.next().unwrap().to_string());
        Card { value, suit }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hand {
    pub hand_size: usize,
    pub cards: Vec<Card>,
}

impl Parsable for Hand {
    fn to_string(&self) -> Option<String> {
        // Sort the cards so that the order is always the same
        let mut s = String::new();
        for card in self.cards.clone() {
            s.push_str(&card.to_string().unwrap());
        }
        Some(s)
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Hand {
    fn new() -> Self {
        Hand {
            hand_size: 2,
            cards: Vec::new(),
        }
    }
    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
        self.cards.sort_by(|a, b| {
            a.value
                .to_string()
                .unwrap()
                .cmp(&b.value.to_string().unwrap())
        });
    }

    fn expand(&mut self) {
        self.hand_size = match self.hand_size {
            2 => 3,
            _ => panic!("Cannot expand this hand size anymore!"),
        }
    }

    fn needs_hole_cards(&self) -> bool {
        self.hand_size > self.cards.len()
    }

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn as_u8(&self) -> Box<[u8]> {
        let mut result = Vec::new();
        for card in self.cards.clone() {
            result.push(card.to_usize().unwrap() as u8);
        }
        result.clone().into_boxed_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Winner {
    Player(usize),
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuctionPokerAction {
    Fold,
    Call,
    Check,
    DealHole(CardIndex, usize), // Card dealt, player index
    DealCommunity(CardIndex),
    Raise(u32, u32), // amount, nearest whole percent
    Bid(u32),        //  representing an auction size from one of the players
    Auction(Winner),
}

impl Parsable for AuctionPokerAction {
    fn to_string(&self) -> Option<String> {
        None
    }

    fn to_usize(&self) -> Option<usize> {
        None
    }
}

impl Into<ActionIndex> for AuctionPokerAction {
    fn into(self) -> ActionIndex {
        todo!();
    }
}

impl From<ActionIndex> for AuctionPokerAction {
    fn from(index: ActionIndex) -> Self {
        todo!();
    }
}

impl Filterable for AuctionPokerAction {}
impl Action for AuctionPokerAction {}

#[derive(Debug, Clone)]
pub struct AuctionPokerState {
    card_bits: u64,
    bids: [Option<u32>; 2],
    player_hands: [Hand; 2],
    community_cards: Vec<Card>,
    pot: u32,
    pips: [u32; 2], // Amount of money each player has put into the pot per betting round
    stacks: [u32; 2],
    raise: Option<u32>, // Cost of the last raise
    active_player: ActivePlayer<AuctionPokerAction>,
}

impl AuctionPokerState {
    fn needs_hole_cards(&self) -> bool {
        self.player_hands[0].needs_hole_cards() || self.player_hands[1].needs_hole_cards()
    }

    fn hole_card_dealer(&self) -> ActivePlayer<AuctionPokerAction> {
        let player_num = match self.player_hands[0].needs_hole_cards() {
            true => 0,
            false => 1,
        };

        let mut cards = Vec::new();
        for i in 0..52 {
            if self.card_bits & (1 << i) == 0 {
                cards.push(AuctionPokerAction::DealHole(i, player_num));
            }
        }

        ActivePlayer::Chance(Categorical::uniform(cards))
    }

    fn next_dealer(&self) -> ActivePlayer<AuctionPokerAction> {
        match self.community_cards.len() {
            0 => self.deal(),
            3 => self.deal(),
            4 => self.deal(),
            5 => self.showdown(),
            _ => panic!("Cannot deal to community when there are this many community cards."),
        }
    }

    fn initial_node() -> ActivePlayer<AuctionPokerAction> {
        let mut cards = Vec::new();
        for i in 0..52 {
            cards.push(AuctionPokerAction::DealHole(i, 0));
        }
        ActivePlayer::Chance(Categorical::uniform(cards))
    }

    fn deal(&self) -> ActivePlayer<AuctionPokerAction> {
        let mut cards = Vec::new();
        for i in 0..52 {
            if self.card_bits & (1 << i) == 0 {
                cards.push(AuctionPokerAction::DealCommunity(i));
            }
        }

        ActivePlayer::Chance(Categorical::uniform(cards))
    }


    fn betting_round(&self, player_num: usize) -> ActivePlayer<AuctionPokerAction> {
        // Amount needed to bet/raise instead of call
        // this represents the total amount of money needed
        let min_raise = match self.raise {
            Some(raise) => raise + self.pips[player_num ^ 1],
            None => BIG_BLIND,
        };

        let mut actions = Vec::new();
        let mut last_rounded = 0;

        let max_raise = self.stacks[player_num].min(self.stacks[player_num ^ 1]);

        for i in min_raise..=max_raise {
            let percent = (i as f32 / self.pot as f32) * 100.0;
            let rounded = percent.round() as u32;
            if rounded == last_rounded {
                continue;
            }
            actions.push(AuctionPokerAction::Raise(i, rounded));
            last_rounded = rounded;
        }

        let all_in = self.stacks[player_num];
        if all_in > 0
            && actions.len() == 0
            && self.stacks[player_num] <= self.stacks[player_num ^ 1]
        {
            let percent = (all_in as f32 / self.pot as f32) * 100.0;
            let rounded = percent.round() as u32;
            actions.push(AuctionPokerAction::Raise(all_in, rounded));
        }

        if self.pips[player_num] == self.pips[player_num ^ 1] {
            // Nobody raised, so we can check
            actions.push(AuctionPokerAction::Check);
        } else {
            // We can always call if the stacks are unequal
            actions.push(AuctionPokerAction::Call);
            actions.push(AuctionPokerAction::Fold);
        }

        ActivePlayer::Player(player_num as u32, actions)
    }

    fn auction(&self) -> ActivePlayer<AuctionPokerAction> {
        let player0_bids = (0..=self.stacks[0])
            .map(|x| AuctionPokerAction::Bid(x))
            .collect::<Vec<_>>();
        let player1_bids = (0..=self.stacks[1])
            .map(|x| AuctionPokerAction::Bid(x))
            .collect::<Vec<_>>();

        match self.bids {
            [None, None] => ActivePlayer::Player(0, player0_bids),
            [Some(_), None] => ActivePlayer::Player(1, player1_bids),
            [Some(bid0), Some(bid1)] => {
                let winner: Winner = if bid0 > bid1 {
                    Winner::Player(0)
                } else if bid1 > bid0 {
                    Winner::Player(1)
                } else {
                    Winner::Tie
                };
                // Hacky way to just force someone to take record the winner
                // by forcing it to be the only action they can take
                ActivePlayer::Player(0, vec![AuctionPokerAction::Auction(winner)])
            }
            _ => panic!("Invalid bids states"),
        }
    }

    /// One of the two players folded
    fn folded(&self, player_num: usize) -> ActivePlayer<AuctionPokerAction> {
        let contribution = STACK_SIZE - self.stacks[player_num];
        let delta = contribution as f32;
        match player_num {
            0 => ActivePlayer::Terminal(vec![-delta, delta]),
            1 => ActivePlayer::Terminal(vec![delta, -delta]),
            _ => panic!("Invalid player number"),
        }
    }

    /// The game is over, determine the winner
    fn showdown(&self) -> ActivePlayer<AuctionPokerAction> {
        let mut player0 = self.player_hands[0].clone();
        let mut player1 = self.player_hands[1].clone();

        for card in &self.community_cards {
            player0.add_card(card.clone());
            player1.add_card(card.clone());
        }

        let player0_hand_len = player0.len();
        let player1_hand_len = player1.len();

        // TODO: probably a bit slow to keep reloading the library
        let hand_ranker = HandRanker::new();

        let player1_rank = match player1_hand_len {
            8 => hand_ranker.rank8(&player1.as_u8()),
            7 => hand_ranker.rank7(&player1.as_u8()),
            _ => panic!("Invalid hand + community length"),
        };

        let player0_rank = match player0_hand_len {
            8 => hand_ranker.rank8(&player0.as_u8()),
            7 => hand_ranker.rank7(&player0.as_u8()),
            _ => panic!("Invalid hand + community length"),
        };

        let contribution0 = STACK_SIZE - self.stacks[0];
        let contribution1 = STACK_SIZE - self.stacks[1];

        let contribution0 = contribution0 as f32;
        let contribution1 = contribution1 as f32;

        // See piazza: extra chip awarded to BB in an odd pot with a tie (BB always
        // second to play)
        let extra_chip = (self.pot % 2) as f32;
        let half_pot = (self.pot as f32 - extra_chip) / 2.0;

        let deltas = match player0_rank.cmp(&player1_rank) {
            Ordering::Greater => vec![contribution1, -contribution1],
            Ordering::Less => vec![-contribution0, contribution0],
            Ordering::Equal => vec![
                contribution0 - half_pot,
                contribution1 - half_pot + extra_chip,
            ],
        };

        ActivePlayer::Terminal(deltas)
    }
}

impl State<AuctionPokerAction> for AuctionPokerState {
    fn new() -> Self {
        AuctionPokerState {
            card_bits: 0,
            bids: [None, None],
            player_hands: [Hand::new(), Hand::new()],
            pot: LITTLE_BLIND + BIG_BLIND,
            community_cards: Vec::new(),
            stacks: [400 - LITTLE_BLIND, 400 - BIG_BLIND],
            pips: [1, 2],
            raise: Some(2),
            active_player: AuctionPokerState::initial_node(),
        }
    }

    fn get_observation(&self, action: &AuctionPokerAction) -> Observation<AuctionPokerAction> {
        match action {
            AuctionPokerAction::Fold => Observation::Public(Information::Action(action.clone())),
            AuctionPokerAction::Call => Observation::Public(Information::Action(action.clone())),
            AuctionPokerAction::Check => Observation::Public(Information::Action(action.clone())),
            AuctionPokerAction::DealHole(_, player_num) => {
                // Share the private information only with the player who got the card
                Observation::Shared(Information::Action(action.clone()), vec![*player_num])
            }
            AuctionPokerAction::DealCommunity(_) => {
                Observation::Public(Information::Action(action.clone()))
            }
            AuctionPokerAction::Raise(_, _) => {
                Observation::Public(Information::Action(action.clone()))
            }
            AuctionPokerAction::Bid(_) => Observation::Private(Information::Action(action.clone())),
            AuctionPokerAction::Auction(_) => {
                Observation::Public(Information::Action(action.clone()))
            }
        }
    }

    fn active_player(&self) -> ActivePlayer<AuctionPokerAction> {
        return self.active_player.clone();
    }

    fn update(&mut self, action: AuctionPokerAction) {
        match action {
            AuctionPokerAction::Fold => {
                let player_num = self.active_player.player_num() as usize;
                self.active_player = self.folded(player_num);
            }
            AuctionPokerAction::Call => {
                let max_pip = self.pips.iter().max().unwrap();

                // Take the pip diff and subtract from both players
                self.stacks = [
                    self.stacks[0] - (max_pip - self.pips[0]),
                    self.stacks[1] - (max_pip - self.pips[1]),
                ];
                // add diffs to pot
                self.pot += (max_pip - self.pips[0]) + (max_pip - self.pips[1]);
                self.pips = [0, 0];
                self.raise = None;

                debug_assert_eq!(self.stacks[0] + self.stacks[1] + self.pot, 2 * STACK_SIZE);
                self.active_player = self.next_dealer();
            }
            AuctionPokerAction::Check => {
                // No change in any of the players stacks, move on

                let player_num = self.active_player.player_num() as usize;
                match player_num {
                    0 => self.active_player = self.betting_round(1),
                    1 => self.active_player = self.next_dealer(),
                    _ => panic!("Invalid player number"),
                }
            }
            AuctionPokerAction::DealHole(card_index, player_num) => {
                let card = Card::from_index(card_index);
                self.player_hands[player_num].add_card(card);
                self.card_bits |= 1 << card_index;
                if self.needs_hole_cards() {
                    self.active_player = self.hole_card_dealer();
                } else {
                    // Start off the next round of betting!
                    self.active_player = self.betting_round(0);
                }
            }
            AuctionPokerAction::DealCommunity(card_index) => {
                self.community_cards.push(Card::from_index(card_index));
                self.card_bits |= 1 << card_index;
                let street = self.community_cards.len();
                let bidding_round_over = self.bids[1].is_some();
                self.active_player = match (street, bidding_round_over) {
                    (0..=2, _) => self.deal(),
                    (3, false) => self.auction(),
                    (3, true) => self.betting_round(0),
                    (4, _) => self.betting_round(0),
                    (5, _) => self.betting_round(0),
                    _ => panic!("Unsure what to do after dealing in this situation")
                }
            }
            AuctionPokerAction::Raise(amount, _) => {
                let player_num = self.active_player().player_num();

                let cost = amount - self.pips[player_num];

                // Opponent bet something - so this is a raise
                if self.pips[player_num ^ 1] > 0 {
                    debug_assert_eq!(amount > self.pips[player_num ^ 1], true);
                    if self.raise.is_some() {
                        debug_assert_eq!(amount >= self.raise.unwrap(), true);
                    }
                    self.raise = Some(amount - self.pips[player_num ^ 1]);
                } else {
                    self.raise = None;
                }

                self.pips[player_num] += cost;
                self.stacks[player_num] -= cost;
                self.pot += cost;

                // Pass the action to the other player
                self.active_player = self.betting_round(player_num ^ 1);
            }

            AuctionPokerAction::Bid(usize) => {
                let player_num = self.active_player().player_num();
                self.bids[player_num] = Some(usize);
                self.active_player = self.auction();
            }

            AuctionPokerAction::Auction(winner) => {
                match winner {
                    Winner::Player(player_num) => {
                        // Loser's bid goes in the pot and is taken from winner
                        self.pot += self.bids[player_num ^ 1].unwrap();
                        self.stacks[player_num] -= self.bids[player_num ^ 1].unwrap();
                        // Winner gets another card!
                        self.player_hands[player_num].expand();
                    }
                    Winner::Tie => {
                        // Both players get another card!
                        self.player_hands[0].expand();
                        self.player_hands[1].expand();
                    }
                }
                // Always needs to deal hole cards after an auction
                self.active_player = self.hole_card_dealer();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chance_transition() {
        let mut state = AuctionPokerState::new();
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 52);
                let action = chance.sample();
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 51);
                let action = chance.sample();
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Chance(chance) => {
                assert_eq!(chance.items().len(), 50);
                let action = chance.sample();
                match action {
                    AuctionPokerAction::DealHole(_, player_num) => {
                        assert_eq!(player_num, 1, "Expected player 1 to get the card next");
                    }
                    _ => panic!("Expected deal player action ."),
                }
                state.update(action);
            }
            _ => panic!("Expected chance transition."),
        }
        // Glass box testing
        assert_eq!(state.card_bits.count_ones(), 3);
    }

    #[test]
    fn test_immediate_fold() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        for _ in 0..4 {
            let active_player = state.active_player();
            match active_player {
                ActivePlayer::Chance(chance) => {
                    let action = chance.sample();
                    state.update(action);
                }
                _ => panic!("Expected chance transition."),
            }
        }
        // First player should be able to fold
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), true);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), true);
                // Cannot check on the first turn with only BB/LB in pot
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Fold);
        // Should be terminal state with player 1 winning LB
        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Terminal(deltas) => {
                assert_eq!(deltas[0], -1.0); // Player 0 loses LB
                assert_eq!(deltas[1], 1.0); // Player 1 wins LB
            }
            x => panic!("Expected terminal state. Got {:?}", x),
        }
    }

    #[test]
    fn test_auction_tie() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::Raise(4, 100));
        state.update(AuctionPokerAction::Call);
        // Deal a bunch of community cards
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        println!("{:?}", state);

        state.update(AuctionPokerAction::Bid(20));
        state.update(AuctionPokerAction::Bid(20));

        assert_eq!(
            state
                .active_player()
                .actions()
                .contains(&AuctionPokerAction::Auction(Winner::Tie)),
            true
        );
    }

    #[test]
    fn test_auction() {
        let mut state = AuctionPokerState::new();
        // Deal four cards
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::Raise(4, 100));
        state.update(AuctionPokerAction::Call);
        // Deal a bunch of community cards
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        println!("{:?}", state);

        match state.active_player() {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
                for i in 0..=396 {
                    assert_eq!(
                        i <= state.stacks[0],
                        true,
                        "Expected i to match the stack size {}. Stack size : {}",
                        i,
                        state.stacks[0]
                    );
                    assert_eq!(
                        actions.contains(&AuctionPokerAction::Bid(i)),
                        true,
                        "Expected bid {} to be available",
                        i
                    );
                }
                assert_eq!(
                    actions.contains(&AuctionPokerAction::Bid(397)),
                    false,
                    "Expected bid 397 to be unavailable"
                );
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Bid(100));
        match state.active_player() {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 1);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), false);
                for i in 0..=396 {
                    assert_eq!(
                        i <= state.stacks[1],
                        true,
                        "Expected i to match the stack size {}. Stack size : {}",
                        i,
                        state.stacks[1]
                    );
                    assert_eq!(
                        actions.contains(&AuctionPokerAction::Bid(i)),
                        true,
                        "Expected bid {} to be available",
                        i
                    );
                }
                assert_eq!(
                    actions.contains(&AuctionPokerAction::Bid(397)),
                    false,
                    "Expected bid 397 to be unavailable"
                );
            }

            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Bid(300));

        // Player 1 should have won the auction
        assert_eq!(
            state
                .active_player()
                .actions()
                .contains(&AuctionPokerAction::Auction(Winner::Player(1))),
            true
        );
    }

    #[test]
    fn test_flop_check_check() {
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(0, 0));
        state.update(AuctionPokerAction::DealHole(2, 0));
        state.update(AuctionPokerAction::DealHole(3, 1));
        state.update(AuctionPokerAction::DealHole(4, 1));
        state.update(AuctionPokerAction::Call);
        state.update(AuctionPokerAction::DealCommunity(5));
        state.update(AuctionPokerAction::DealCommunity(6));
        state.update(AuctionPokerAction::DealCommunity(7));
        state.update(AuctionPokerAction::Bid(0));
        state.update(AuctionPokerAction::Bid(1));
        state.update(AuctionPokerAction::Auction(Winner::Player(1)));
        state.update(AuctionPokerAction::DealHole(8, 1));

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 0);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), true);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }

        state.update(AuctionPokerAction::Check);

        let active_player = state.active_player();
        match active_player {
            ActivePlayer::Player(player, actions) => {
                assert_eq!(player, 1);
                assert_eq!(actions.contains(&AuctionPokerAction::Fold), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Call), false);
                assert_eq!(actions.contains(&AuctionPokerAction::Check), true);
            }
            x => panic!("Expected player transition. Got {:?}", x),
        }
    }

    #[test]
    fn test_showdown() {
        // Should win the pot when dealt the nuts
        let mut state = AuctionPokerState::new();
        state.update(AuctionPokerAction::DealHole(Card::new("Ah").to_usize().unwrap(), 0));
        state.update(AuctionPokerAction::DealHole(Card::new("Ac").to_usize().unwrap(), 0));
        state.update(AuctionPokerAction::DealHole(Card::new("2c").to_usize().unwrap(), 1));
        state.update(AuctionPokerAction::DealHole(Card::new("2h").to_usize().unwrap(), 1));

        // First betting round (pre-flop)
        state.update(AuctionPokerAction::Raise(9, 1337));
        state.update(AuctionPokerAction::Call);

        // pot = 18

        // Flop dealt
        state.update(AuctionPokerAction::DealCommunity(Card::new("Ad").to_usize().unwrap()));
        state.update(AuctionPokerAction::DealCommunity(Card::new("As").to_usize().unwrap()));
        state.update(AuctionPokerAction::DealCommunity(Card::new("2d").to_usize().unwrap()));

        // Auction starts
        state.update(AuctionPokerAction::Bid(50));
        state.update(AuctionPokerAction::Bid(25));

        // Make sure that player 0 won!
        // pot = 18 + 25 = 43 (9 contributed by player 1)
        assert_eq!(state.active_player().actions().contains(&AuctionPokerAction::Auction(Winner::Player(0))), true);

        state.update(AuctionPokerAction::Auction(Winner::Player(0)));
        state.update(AuctionPokerAction::DealHole(Card::new("2c").to_usize().unwrap(), 0));

        // Make sure that we have moved on to the next betting round!
        // By checking if it's the first player and we're allowed to raise 
        assert_eq!(state.active_player().actions().iter().any(|x| matches!(x, AuctionPokerAction::Raise(_,_))), true);
        assert_eq!(state.active_player().player_num() == 0, true);

        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::Check);

        // Make sure we're in the card dealing round
        assert!(state.active_player().actions().iter().any( |x| matches!( x, AuctionPokerAction::DealCommunity(_))));

        // Turn dealt
        state.update(AuctionPokerAction::DealCommunity(Card::new("Qc").to_usize().unwrap()));

        // Make sure that we have moved on to the next betting round!
        println!(" Active player {:?}" , state.active_player());
        assert_eq!(state.active_player().actions().iter().any(|x| matches!(x, AuctionPokerAction::Raise(_,_))), true);
        assert_eq!(state.active_player().player_num() == 0, true);

        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::Check);

        // Make sure we're in the card dealing round
        assert!(state.active_player().actions().iter().any( |x| matches!( x, AuctionPokerAction::DealCommunity(_))));


        // River dealt
        state.update(AuctionPokerAction::DealCommunity(Card::new("3c").to_usize().unwrap()));

        // Make sure that we have moved on to the next betting round!
        // By checking if it's the first player and we're allowed to raise 
        assert_eq!(state.active_player().actions().iter().any(|x| matches!(x, AuctionPokerAction::Raise(_,_))), true);
        assert_eq!(state.active_player().player_num() == 0, true);


        state.update(AuctionPokerAction::Check);
        state.update(AuctionPokerAction::Check);

        assert!(matches!( state.active_player(), ActivePlayer::Terminal(_)));
        if let ActivePlayer::Terminal(deltas) = state.active_player() {
            assert!((deltas[0] - 9.0) < 0.00001);
            assert!((deltas[1] - -9.0) < 0.00001);
        }
    }

    #[test]
    fn test_sample_playthrough() {
        // Go through while sampling chance and fixing the actions
        // for each player
    }

    #[test]
    fn test_reraise() {
        // Make sure that reraising works
    }

    #[test]
    fn test_card_coherence() {
        let card_str = "9d";
        let card_interpreted = Card::from_index(Card::new(card_str).to_usize().unwrap());
        assert_eq!(card_interpreted, Card::new(card_str));
        assert_eq!(card_interpreted.to_string().unwrap(), card_str.to_owned());
    }
}
