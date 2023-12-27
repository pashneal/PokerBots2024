mod constants;

use gtcogs::{goofspiel, Goofspiel, OuterMCCFR};
use rand::{rngs::SmallRng, SeedableRng};

pub fn main() -> () {
    let g = Goofspiel::new(5, goofspiel::Scoring::ZeroSum);
    let mut mc = OuterMCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(2);
    mc.compute_rng(10000000, 0.6, &mut rng);

    mc.write_to("goofspiel");
}
