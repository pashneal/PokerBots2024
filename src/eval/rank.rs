use crate::game_logic::action::Parsable;
use crate::implementations::auction::Card;
use libloading::{Library, Symbol};
use std::time::{Duration, Instant};

pub struct HandRanker {
    library: Library,
}

impl HandRanker {
    pub fn new() -> HandRanker {
        unsafe {
            let library = Library::new("./librank.so").unwrap();
            HandRanker { library }
        }
    }

    pub fn rank7(&self, cards: &[u8]) -> u32 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u8) -> u32> =
                self.library.get(b"get_rank7").unwrap();
            func(
                cards[0], cards[1], cards[2], cards[3], cards[4], cards[5], cards[6],
            )
        }
    }

    pub fn rank8(&self, cards: &[u8]) -> u32 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u8, u8) -> u32> =
                self.library.get(b"get_rank8").unwrap();
            func(
                cards[0], cards[1], cards[2], cards[3], cards[4], cards[5], cards[6], cards[7]
            )
        }
    }

    pub fn rollout_2_7(&self, cards : &[u8], iterations : u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u32) -> f64> =
                self.library.get(b"rollout_2_7").unwrap();
            func(cards[0], cards[1], iterations)
        }
    }
    pub fn rollout_2_8(&self, cards : &[u8], iterations : u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u32) -> f64> =
                self.library.get(b"rollout_2_8").unwrap();
            func(cards[0], cards[1], iterations)
        }
    }

    pub fn rollout_bid_win(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_bid_win").unwrap();
            func(hand[0], hand[1], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }
    pub fn rollout_bid_loss(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_bid_loss").unwrap();
            func(hand[0], hand[1], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }

    pub fn rollout_bid_tie(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_bid_tie").unwrap();
            func(hand[0], hand[1], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }

    pub fn rollout_flop_won(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_flop_won").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }

    pub fn rollout_flop_lost(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_flop_lost").unwrap();
            func(hand[0], hand[1], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }


    pub fn rollout_flop_tie(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_flop_tie").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }

    pub fn rollout_turn_won(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_turn_won").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], community_cards[3], iterations)
        }
    }

    pub fn rollout_turn_lost(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_turn_lost").unwrap();
            func(hand[0], hand[1], community_cards[0], community_cards[1], community_cards[2], iterations)
        }
    }

    pub fn rollout_turn_tie(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_turn_tie").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], community_cards[3], iterations)
        }
    }

    pub fn rollout_river_won(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(u8, u8, u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_river_won").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], community_cards[3], community_cards[4], iterations)
        }
    }

    pub fn rollout_river_lost(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(
                u8, u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_river_lost").unwrap();
            func(hand[0], hand[2], community_cards[0], community_cards[1], community_cards[2], community_cards[3], community_cards[4], iterations)
        }
    }

    pub fn rollout_river_tie(&self, hand: &[u8] ,  community_cards : &[u8], iterations: u32) -> f64 {
        unsafe {
            let func: Symbol<unsafe extern "C" fn(
                u8, u8, u8, u8, u8, u8, u8, u8, u32) -> f64> =
                self.library.get(b"rollout_river_tie").unwrap();
            func(hand[0], hand[1], hand[2], community_cards[0], community_cards[1], community_cards[2], community_cards[3], community_cards[4], iterations)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tie_win_loss_strengths() {
        let hand_ranker = HandRanker::new();
        let hand = [
            Card::new("Kc").to_usize().unwrap() as u8, 
            Card::new("Kd").to_usize().unwrap() as u8
        ];
        let community_cards = [
            Card::new("2h").to_usize().unwrap() as u8, 
            Card::new("3s").to_usize().unwrap() as u8, 
            Card::new("4h").to_usize().unwrap() as u8
        ];
        let iterations = 10_000;
        let time = Instant::now(); 
        let tie_strength = hand_ranker.rollout_bid_tie(&hand, &community_cards, iterations);
        let win_strength = hand_ranker.rollout_bid_win(&hand, &community_cards, iterations);
        let loss_strength = hand_ranker.rollout_bid_loss(&hand, &community_cards, iterations);
        println!("Time: {:?}", time.elapsed());
        println!("Tie: {}", tie_strength);
        println!("Win: {}", win_strength);
        println!("Loss: {}", loss_strength);
        assert!(win_strength > tie_strength);
        assert!(win_strength > loss_strength);
        assert!(loss_strength < tie_strength);
    }
    #[test] 
    fn test_rollout_bid() {
        let hand_ranker = HandRanker::new();
        let hand = [
            Card::new("Kc").to_usize().unwrap() as u8, 
            Card::new("Kd").to_usize().unwrap() as u8
        ];
        let community_cards = [
            Card::new("Kh").to_usize().unwrap() as u8, 
            Card::new("Qs").to_usize().unwrap() as u8, 
            Card::new("4h").to_usize().unwrap() as u8
        ];
        let iterations = 10_000;
        let strong = hand_ranker.rollout_bid_win(&hand, &community_cards, iterations);
        let weak = hand_ranker.rollout_bid_loss(&hand, &community_cards, iterations);
        println!("Strong (bid): {}", strong);
        println!("Weak (bid): {}", weak);
        assert!(strong > weak);

    }
    #[test]
    fn rollout_with_8_better_than_7_straight() {
        let hand_ranker = HandRanker::new();
        let card1 = Card::new("Tc").to_usize().unwrap() as u8;
        let card2 = Card::new("9c").to_usize().unwrap() as u8;
        let cards = [card1, card2];
        let iterations = 100_000;
        let weak = hand_ranker.rollout_2_7(&cards, iterations);
        let strong = hand_ranker.rollout_2_8(&cards, iterations);
        println!("Weak (straight): {}", weak);
        println!("Strong (straight): {}", strong);
        assert!(weak < strong);
    }
    #[test]
    fn rollout_with_8_better_than_7_kings() {
        let hand_ranker = HandRanker::new();
        let card1 = Card::new("Kc").to_usize().unwrap() as u8;
        let card2 = Card::new("Kd").to_usize().unwrap() as u8;
        let cards = [card1, card2];
        let iterations = 100_000;
        let time = std::time::Instant::now();
        let weak = hand_ranker.rollout_2_7(&cards, iterations);
        let strong = hand_ranker.rollout_2_8(&cards, iterations);
        println!("Weak (kings): {}", weak);
        println!("Strong (kings): {}", strong);
        assert!(weak < strong);
    }


    #[test]
    fn rollout_with_8_better_than_7_aces() {
        let hand_ranker = HandRanker::new();
        let card1 = Card::new("Ad").to_usize().unwrap() as u8;
        let card2 = Card::new("Ah").to_usize().unwrap() as u8;
        let cards = [card1, card2];
        let iterations = 100_000;
        let weak = hand_ranker.rollout_2_7(&cards, iterations);
        let strong = hand_ranker.rollout_2_8(&cards, iterations);
        println!("Weak: {}", weak);
        println!("Strong: {}", strong);
        assert!(weak < strong);
    }

    #[test]
    fn can_call_rollout_function() {
        let hand_ranker = HandRanker::new();
        let card1 = Card::new("Kc").to_usize().unwrap() as u8;
        let card2 = Card::new("Jd").to_usize().unwrap() as u8;
        let cards = [card1, card2];
        let iterations = 10_000;
        // Time it!
        let start = std::time::Instant::now();
        let result = hand_ranker.rollout_2_7(&cards, iterations);
        let end = std::time::Instant::now();
        println!("Result: {}", result);
        println!("Time: {:?}", end - start);
        assert!(result > 0.0); // Not necessarily true, but highly highly unlikely
        assert!(result <= 1.0);
    }

    #[test]
    fn test_load_libso() {
        let hand_ranker = HandRanker::new();
        let cards = [1, 2, 3, 4, 5, 6, 7];
        let rank = hand_ranker.rank7(&cards);
        assert_eq!(rank, 7440);
    }

    #[test]
    fn test_royal_flush_8() {
        let hand_ranker = HandRanker::new();
        // test with royal flush so we are sure it is working (only one possible
        // best hand in set of 8 if royal flush)
        let cards  = [
            Card::new("2d"),
            Card::new("4d"),
            Card::new("Kc"),
            Card::new("Qc"),
            Card::new("Jc"),
            Card::new("3d"),
            Card::new("Tc"),
            Card::new("Ac"),
        ];

        let cards = cards
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();
        let rank = hand_ranker.rank8(&cards);
        let cards7  = [
            Card::new("4h"),
            Card::new("Ac"),
            Card::new("9d"),
            Card::new("Kc"),
            Card::new("Qc"),
            Card::new("Jc"),
            Card::new("Tc"),
        ];

        let cards7 = cards7
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();
        let rank7 = hand_ranker.rank7(&cards7);
        assert_eq!(rank, rank7);
    }

    #[test]
    fn test_rank8() {
        let hand_ranker = HandRanker::new();
        let cards = [5, 6, 7, 8, 9, 10, 11, 12];
        let rank = hand_ranker.rank8(&cards);
        assert_eq!(rank, 7427);
    }

    #[test]
    fn test_card_interpretation() {
        let hand_ranker = HandRanker::new();
        let royal_flush = [
            Card::new("Kc"),
            Card::new("Qc"),
            Card::new("Jc"),
            Card::new("Tc"),
            Card::new("2d"),
            Card::new("3d"),
            Card::new("Ac"),
        ];
        let royal_flush = royal_flush
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();

        let straight_flush = [
            Card::new("2c"),
            Card::new("3c"),
            Card::new("4c"),
            Card::new("5c"),
            Card::new("6c"),
            Card::new("7c"),
            Card::new("8c"),
        ];
        let straight_flush = straight_flush
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();

        let two_pair = [
            Card::new("2c"),
            Card::new("2d"),
            Card::new("3c"),
            Card::new("3d"),
            Card::new("4c"),
            Card::new("4d"),
            Card::new("5c"),
        ];
        let two_pair = two_pair
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();

        let high_card = [
            Card::new("2c"),
            Card::new("3d"),
            Card::new("7h"),
            Card::new("5d"),
            Card::new("6c"),
            Card::new("9d"),
            Card::new("Ac"),
        ];
        let high_card = high_card
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();

        let lower_high_card = [
            Card::new("2c"),
            Card::new("3d"),
            Card::new("7h"),
            Card::new("5d"),
            Card::new("6c"),
            Card::new("9s"),
            Card::new("Kc"),
        ];
        let lower_high_card = lower_high_card
            .iter()
            .map(|c| c.to_usize().unwrap() as u8)
            .collect::<Vec<u8>>();

        // Lower high card should lose to higher high card
        assert!(hand_ranker.rank7(&high_card) > hand_ranker.rank7(&lower_high_card));
        // Two pair should beat high card
        assert!(hand_ranker.rank7(&two_pair) > hand_ranker.rank7(&high_card));
        // Two pair should beat lower high card
        assert!(hand_ranker.rank7(&two_pair) > hand_ranker.rank7(&lower_high_card));
        // Straight flush should beat two pair
        assert!(hand_ranker.rank7(&straight_flush) > hand_ranker.rank7(&two_pair));
        // Straight flush should beat high card
        assert!(hand_ranker.rank7(&straight_flush) > hand_ranker.rank7(&high_card));
        // Straight flush should beat lower high card
        assert!(hand_ranker.rank7(&straight_flush) > hand_ranker.rank7(&lower_high_card));
        // Royal flush should beat straight flush
        assert!(hand_ranker.rank7(&royal_flush) > hand_ranker.rank7(&straight_flush));
        // Royal flush should beat two pair
        assert!(hand_ranker.rank7(&royal_flush) > hand_ranker.rank7(&two_pair));
        // Royal flush should beat high card
        assert!(hand_ranker.rank7(&royal_flush) > hand_ranker.rank7(&high_card));
        // Royal flush should beat lower high card
        assert!(hand_ranker.rank7(&royal_flush) > hand_ranker.rank7(&lower_high_card));
        // If these tests pass, you're probably using the SKPokerEval library correctly!
    }
}
