pub mod regret;
pub mod blueprint;
pub use regret::*;
pub use blueprint::*;

use dashmap::DashMap;

pub type CondensedInfoSet = u64;
pub type PolicyDistribution = Vec<f32>;
pub type RegretDistribution = Vec<f32>;
pub type PolicyMap = DashMap<CondensedInfoSet, PolicyDistribution>;
pub type RegretMap = DashMap<CondensedInfoSet, RegretDistribution>;
