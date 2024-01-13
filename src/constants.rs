pub const HOT_ENCODING_SIZE: usize = 30;
pub const ACTION_SPACE_SIZE: usize = 10;
pub const MAX_GAME_DEPTH: usize = 1000;
pub const NUM_REGULAR_PLAYERS: usize = 2;
pub const CHANCE_PLAYERS: usize = 1;
pub const TOTAL_PLAYERS: usize = NUM_REGULAR_PLAYERS + CHANCE_PLAYERS;

pub const BIG_BLIND: u32 = 2;
pub const LITTLE_BLIND: u32 = 1;
pub const MIN_BET_AMOUNT: u32 = BIG_BLIND;
pub const STACK_SIZE: u32 = 400;
pub const MAX_POT: u32 = 2 * STACK_SIZE;

pub const EV_ITERATIONS: u32 = 10_000;
