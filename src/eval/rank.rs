use libloading::{Library, Symbol};
use crate::implementations::auction::Card;
use crate::game_logic::action::Parsable;

pub struct HandRanker {
     library : Library,
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
            let func: Symbol<unsafe extern fn(u8,u8,u8,u8,u8,u8,u8) -> u32> = self.library.get(b"get_rank").unwrap();
            func(cards[0], cards[1], cards[2], cards[3], cards[4], cards[5], cards[6])
        }
    }

    pub fn rank8(&self, cards: &[u8]) -> u32 {
        unsafe {
            let func: Symbol<unsafe extern fn(u8,u8,u8,u8,u8,u8,u8) -> u32> = self.library.get(b"get_rank").unwrap();
            let mut max_rank = 0;
            for i in 0..8 {
                let mut cards8 = cards.iter().map(|c| *c).collect::<Vec<u8>>();
                cards8.swap(i, 7);
                let rank = func(cards8[0], cards8[1], cards8[2], cards8[3], cards8[4], cards8[5], cards8[6]);
                if rank > max_rank {
                    max_rank = rank;
                }
            }
            max_rank
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_libso() {
        let hand_ranker = HandRanker::new();
        let cards = [1,2,3,4,5,6,7];
        let rank = hand_ranker.rank7(&cards);
        assert_eq!(rank, 7440);
    }

    #[test]
    fn test_rank8() {
        let hand_ranker = HandRanker::new();
        let cards = [5,6,7,8,9,10,11,12];
        let rank = hand_ranker.rank8(&cards);
        assert_eq!(rank, 7427);
    }

    #[test]
    fn test_card_interpretation() {
        let hand_ranker = HandRanker::new();
        let royal_flush = [
            Card::new("Ac"),
            Card::new("Kc"),
            Card::new("Qc"),
            Card::new("Jc"),
            Card::new("Tc"),
            Card::new("2d"),
            Card::new("3d"),
        ];
        let royal_flush = royal_flush.iter().map(|c| c.to_usize().unwrap() as u8).collect::<Vec<u8>>();


        let straight_flush = [
            Card::new("2c"),
            Card::new("3c"),
            Card::new("4c"),
            Card::new("5c"),
            Card::new("6c"),
            Card::new("7c"),
            Card::new("8c"),
        ];
        let straight_flush = straight_flush.iter().map(|c| c.to_usize().unwrap() as u8).collect::<Vec<u8>>();

        let two_pair = [
            Card::new("2c"),
            Card::new("2d"),
            Card::new("3c"),
            Card::new("3d"),
            Card::new("4c"),
            Card::new("4d"),
            Card::new("5c"),
        ]; 
        let two_pair = two_pair.iter().map(|c| c.to_usize().unwrap() as u8).collect::<Vec<u8>>();


        let high_card = [
            Card::new("2c"),
            Card::new("3d"),
            Card::new("7h"),
            Card::new("5d"),
            Card::new("6c"),
            Card::new("9d"),
            Card::new("Ac"),
        ];
        let high_card = high_card.iter().map(|c| c.to_usize().unwrap() as u8).collect::<Vec<u8>>();

        let lower_high_card = [
            Card::new("2c"),
            Card::new("3d"),
            Card::new("7h"),
            Card::new("5d"),
            Card::new("6c"),
            Card::new("9s"),
            Card::new("Kc"),
        ];
        let lower_high_card = lower_high_card.iter().map(|c| c.to_usize().unwrap() as u8).collect::<Vec<u8>>();

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
