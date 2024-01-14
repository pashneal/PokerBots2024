use std::collections::HashMap;

use crate::game_logic::strategy::CondensedInfoSet;
use crate::game_logic::strategy::PolicyDistribution;
use crate::game_logic::strategy::RegretDistribution;
use crate::game_logic::strategy::RegretMap;
use crate::game_logic::strategy::PolicyMap;


use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct BlueprintStrategy {
    policies : Vec<HashMap<CondensedInfoSet, PolicyDistribution>>,
}



