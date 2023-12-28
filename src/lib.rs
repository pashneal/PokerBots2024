extern crate bit_set;
extern crate hashbrown;
extern crate rand;

mod action;
mod constants;
mod distribution;
mod game;
pub mod goofspiel;
mod history;
mod mccfr;
mod state;
mod strategy;
mod visibility;

pub use self::action::{HotEncoding, IntoHotEncoding};
pub use self::constants::HOT_ENCODING_SIZE;
pub use self::distribution::Categorical;
pub use self::game::Game;
pub use self::mccfr::MCCFR;

pub type ActionIndex = u32;

impl IntoHotEncoding for ActionIndex {
    fn encoding(self) -> HotEncoding {
        let mut v = vec![false; HOT_ENCODING_SIZE];
        v[self as usize] = true;
        v
    }
}

pub type Utility = f64;
