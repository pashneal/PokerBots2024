mod constants;

use gtcogs::{goofspiel, Game, MCCFR};
use crate::goofspiel::{GoofspielAction, GoofspielState};
use rand::{rngs::SmallRng, SeedableRng};

pub fn main() -> () {
    let g = Game::<GoofspielAction, GoofspielState>::new();

    let mut mc = MCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(2);
    mc.run_iterations(10000000, 0.6, &mut rng);

    mc.write_to("goofspiel");
}
