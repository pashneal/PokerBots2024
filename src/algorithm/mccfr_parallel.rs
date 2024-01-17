use crate::algorithm::mccfr::MCCFR;
use crate::constants::*;
use crate::game_logic::action::{Action, GameMapper};
use crate::game_logic::game::Game;
use crate::game_logic::state::State;
use crate::game_logic::strategy::RegretStrategy;
use rand::{rngs::SmallRng, SeedableRng};
use std::marker::{Send, Sync};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MCCFRParallel<A: Action, S: State<A>> {
    runners: Vec<MCCFR<A, S>>,
    threads: usize,
    strategies: Vec<Arc<RegretStrategy>>,
    file_name: Option<String>,
}

impl<A: Action + Sync + Send + 'static, S: State<A> + Send + 'static> MCCFRParallel<A, S> {
    pub fn new(threads: usize, file_name: Option<&str>) -> MCCFRParallel<A, S> {
        let mut runners = Vec::new();
        let strategies = vec![
            Arc::new(RegretStrategy::default()),
            Arc::new(RegretStrategy::default()),
        ];
        for _ in 0..threads {
            runners.push(MCCFR::new(Game::<A, S>::new(), strategies.clone()));
        }
        MCCFRParallel {
            runners,
            threads,
            strategies: strategies.clone(),
            file_name : file_name.map(|s| s.to_string()),
        }
    }

    pub fn run_iterations(&mut self, iterations: usize, epsilon: f32) {
        // Save intermediate results
        // makes sure to pick a  good batch size
        let batch_size = 1000;

        // Total intermediate iterations 
        let total_batches = iterations / batch_size;

        for i in 0..total_batches {
            let iterations = batch_size;
        
            let mut thread_iters = vec![iterations / self.threads; self.threads];
            for i in 0..(iterations % self.threads) {
                thread_iters[i] += 1;
            }
            let mut threads = Vec::new();
            for i in 0..self.threads {
                let mut runner = self.runners[i].clone();
                let iters = thread_iters[i];
                threads.push(std::thread::Builder::new().stack_size(100*1024*1024).spawn(move || {
                    let mut rng = SmallRng::from_rng(&mut rand::thread_rng()).unwrap();
                    runner.run_iterations(iters, epsilon, &mut rng);
                    runner
                }).unwrap());
            }
            for thread in threads {
                let runner = thread.join().unwrap();
                self.runners.push(runner);
            }
            if let Some(file_name) = &self.file_name {
                self.write_to(file_name);
            } else {
                println!("No file name provided, not saving");
            }

        }
    }
    pub fn write_to(&self, file_name: &str) {
        for (i, strategy) in self.strategies.iter().enumerate() {
            let file = format!("{}{}", file_name.to_owned(), format!("_p{}.json", i));
            let game_mapper: GameMapper<A> = GameMapper::new(None);
            strategy.save_table_json(&file, &game_mapper);
        }
    }
}
