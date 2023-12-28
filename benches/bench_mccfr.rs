#![feature(test)]
extern crate gtcogs;
extern crate rand;
extern crate test;

use gtcogs::{goofspiel, MCCFR};
use crate::goofspiel::{GoofspielAction, GoofspielState};
use gtcogs::Game;
use rand::{rngs::SmallRng, SeedableRng};
use test::Bencher;

#[bench]
fn bench_os_mccfr_goofspiel4(b: &mut Bencher) {
    let g = Game::<GoofspielAction, GoofspielState>::new();
    let mut mc = MCCFR::new(g);
    let mut rng = SmallRng::seed_from_u64(1);
    b.iter(|| mc.run_iterations(1000, 0.6, &mut rng));
}

