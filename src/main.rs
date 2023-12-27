mod constants;

use gtcogs::{goofspiel, Goofspiel, MCCFR};
use rand::{rngs::SmallRng, SeedableRng};

pub fn main() -> () {
    let g = Goofspiel::new(5, goofspiel::Scoring::ZeroSum);
    let mut mc = MCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(2);
    mc.run_iterations(10000000, 0.6, &mut rng);

    mc.write_to("goofspiel");
}
