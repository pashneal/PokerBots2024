use crate::constants::*;
use std::fmt::Debug;
use std::hash::Hash;
pub use std::ops::RangeInclusive as StdRange;

pub type ActionIndex = u8;
pub trait Action: Clone + Debug + Filterable + Into<ActionIndex> + From<ActionIndex> {
    fn max_index() -> ActionIndex {
        std::u8::MAX
    }

    fn index(&self) -> ActionIndex {
        unimplemented!()
    }
}

pub type ActionFilter<A> = (Filter<A>, A);

#[derive(Debug, Clone)]
pub struct ActionMapper<A: Action> {
    filters: Vec<ActionFilter<A>>,
}

impl<A: Action> ActionMapper<A> {
    pub fn new() -> Self {
        ActionMapper {
            filters: Vec::new(),
        }
    }
    pub fn add_filter(&mut self, filter: Filter<A>, action: A) {
        self.filters.push((filter, action));
    }

    pub fn map_and_index(&self, action: A) -> (A, ActionIndex) {
        for (index, (filter, mapped_action)) in self.filters.iter().enumerate() {
            if filter.accepts(&action) {
                return (mapped_action.clone(), index as ActionIndex);
            }
        }
        panic!(
            "No filter matched action, check that your filters span the entire action space!! {:?}",
            action
        );
    }

    /// Map an action to a new action
    /// Precondition:
    /// all actions must map to
    /// the same action index after filtering
    pub fn map(&self, action: A) -> A {
        for (filter, mapped_action) in &self.filters {
            if filter.accepts(&action) {
                debug_assert!({
                    let mapped_action_index: ActionIndex = mapped_action.clone().into();
                    let action_index: ActionIndex = action.clone().into();
                    mapped_action_index == action_index
                });
                return mapped_action.clone();
            }
        }
        panic!(
            "No filter matched action, check that your filters span the entire action space!! {:?}",
            action
        );
    }

    pub fn to_index(&self, action: A) -> ActionIndex {
        for (index, (filter, _)) in self.filters.iter().enumerate() {
            if filter.accepts(&action) {
                return index as ActionIndex;
            }
        }
        panic!(
            "No filter matched action, check that your filters span the entire action space!! {:?}",
            action
        );
    }

    pub fn num_groups(&self) -> usize {
        self.filters.len()
    }
}

/// May contain a filter for each depth of the game
/// if no filter is present for a given depth, actions
/// are mapped to themselves
#[derive(Debug, Clone)]
pub struct GameMapper<A: Filterable + Action> {
    depth_specific_maps: Vec<Option<ActionMapper<A>>>,
    recall_depth: usize,
    max_encoding_size: usize,
}

/// TODO: The indexing is weird (don't know if game_mapper indexes correctly)
impl<A: Filterable + Action> GameMapper<A> {
    ///  Create a GameMapper with no default mapping (passes all actions through)
    ///  recall_depth determines how many states will be
    ///  outputted by a HotEncoding
    pub fn new(recall_depth: Option<usize>) -> Self {
        let recall_depth = recall_depth.unwrap_or(MAX_GAME_DEPTH);
        GameMapper {
            depth_specific_maps: vec![None; MAX_GAME_DEPTH],
            recall_depth,
            max_encoding_size: HOT_ENCODING_SIZE,
        }
    }
    /// Create a GameMapper with a given default mapping for all depths
    pub fn from_default(default_map: ActionMapper<A>, recall_depth: Option<usize>) -> Self {
        let recall_depth = recall_depth.unwrap_or(MAX_GAME_DEPTH);
        let encoding_size = default_map.num_groups();
        GameMapper {
            depth_specific_maps: vec![Some(default_map); MAX_GAME_DEPTH],
            recall_depth,
            max_encoding_size: encoding_size,
        }
    }

    /// Create a GameMapper to operate a specific depth of the game
    pub fn update_depth(&mut self, mapper: Option<ActionMapper<A>>, depth: usize) {
        self.depth_specific_maps[depth] = mapper;
        // If there is a mapper, then we need to update the max encoding size
        self.max_encoding_size = 0;
        for mapper in &self.depth_specific_maps {
            match mapper {
                Some(mapper) => {
                    if (mapper.num_groups() > self.max_encoding_size) {
                        self.max_encoding_size = mapper.num_groups();
                    }
                }
                None => self.max_encoding_size = HOT_ENCODING_SIZE,
            }
        }
    }

    pub fn map_and_index(&self, action: A, depth: usize, index: ActionIndex) -> (A, ActionIndex) {
        let mapper = &self.depth_specific_maps[depth];
        match mapper {
            Some(mapper) => mapper.map_and_index(action),
            None => (action, index),
        }
    }

    pub fn map_action(&self, action: A, depth: usize) -> A {
        // TODO: since this is a pure function we can memoize it
        //       for speed improvements
        let mapper = &self.depth_specific_maps[depth];
        match mapper {
            Some(mapper) => mapper.map(action),
            None => action,
        }
    }

    pub fn map_actions(&self, actions: &Vec<A>, depth: usize) -> Vec<A> {
        let mapper = &self.depth_specific_maps[depth];
        let mapped = match mapper {
            Some(mapper) => actions
                .iter()
                .map(|action| mapper.map(action.clone()))
                .collect(),
            None => actions.clone(),
        };

        println!("mapped actions: {:?}", mapped);
        // Group by action index while preserving order
        let max = A::max_index();
        let mut grouped: Vec<Vec<A>> = vec![vec![]; max as usize];
        for action in mapped {
            grouped[action.index() as usize].push(action);
        }
        // Take the median action from each group if the group is non-empty
        let mut median_actions: Vec<A> = vec![];
        for group in grouped {
            if group.len() > 0 {
                let median_index = group.len() / 2;
                median_actions.push(group[median_index].clone());
            }
        };
        median_actions
    }

    pub fn encoding_size(&self) -> usize {
        self.max_encoding_size
    }
}

#[derive(Debug, Clone)]
pub struct Clause<T>
where
    T: Parsable,
{
    pub left: Box<Filter<T>>,
    pub right: Box<Filter<T>>,
}

impl<T> Clause<T>
where
    T: Parsable,
{
    pub fn new(left: Filter<T>, right: Filter<T>) -> Self {
        Clause {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

pub trait Parsable: Clone + Debug + PartialEq {
    fn to_string(&self) -> Option<String>;
    fn to_usize(&self) -> Option<usize>;
}

pub trait Filterable: Parsable {
    fn filter(list: &Vec<Self>, primitive: &Primitive<Self>) -> Vec<Self> {
        match primitive {
            Primitive::Raw(raw) => list.iter().filter(|x| *x == raw).cloned().collect(),
            Primitive::Regex(details) => {
                let re = regex::Regex::new(&details.regex).unwrap();
                list.iter()
                    .filter(|x| match x.to_string() {
                        Some(s) => re.is_match(&s),
                        None => false,
                    })
                    .cloned()
                    .collect()
            }
            Primitive::Range(details) => list
                .iter()
                .filter(|x| match x.to_usize() {
                    Some(n) => details.range.contains(&n),
                    None => false,
                })
                .cloned()
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegexQuery {
    pub regex: String,
}

#[derive(Debug, Clone)]
pub struct RangeQuery {
    pub range: StdRange<usize>,
}

#[derive(Debug, Clone)]
pub enum Primitive<T>
where
    T: Parsable,
{
    Raw(T),
    Regex(RegexQuery),
    Range(RangeQuery),
}

#[derive(Debug, Clone)]
pub enum Filter<T>
where
    T: Parsable,
{
    And(Clause<T>),
    Or(Clause<T>),
    Not(Box<Filter<T>>),
    BaseCase(Primitive<T>),
}

impl<T> Filter<T>
where
    T: Filterable,
{
    pub fn and(self, other: Filter<T>) -> Self {
        Filter::And(Clause::new(self, other))
    }

    pub fn or(self, other: Filter<T>) -> Self {
        Filter::Or(Clause::new(self, other))
    }

    pub fn new(raw: T) -> Self {
        Filter::BaseCase(Primitive::Raw(raw))
    }

    pub fn regex(regex: &str) -> Self {
        Filter::BaseCase(Primitive::Regex(RegexQuery {
            regex: regex.to_string(),
        }))
    }

    pub fn range(range: StdRange<usize>) -> Self {
        Filter::BaseCase(Primitive::Range(RangeQuery { range }))
    }

    pub fn not(self) -> Self {
        Filter::Not(Box::new(self))
    }

    pub fn apply_on(&self, list: &Vec<T>) -> Vec<T> {
        match self {
            Filter::And(clause) => {
                let left = clause.left.apply_on(list);
                let right = clause.right.apply_on(list);
                left.into_iter().filter(|x| right.contains(x)).collect()
            }
            Filter::Or(clause) => {
                let left = clause.left.apply_on(list);
                let right = clause.right.apply_on(list);
                left.into_iter().chain(right.into_iter()).collect()
            }
            Filter::Not(filter) => {
                let filtered = filter.apply_on(list);
                list.iter()
                    .filter(|x| !filtered.contains(x))
                    .cloned()
                    .collect()
            }
            Filter::BaseCase(primitive) => Filterable::filter(list, primitive),
        }
    }

    pub fn accepts(&self, raw: &T) -> bool {
        self.apply_on(&vec![raw.clone()]).len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::auction::{AuctionPokerAction, RelativeSize};
    use std::collections::HashSet;

    #[test]
    pub fn test_default_behavior() {
        // Should map elements that have the same ActionIndex
        // to the median of the inputs when there is no
        // custom ActionMapper provided
        //
        // median here is a bit ill-defined for Fold, Call, Check and even numbered groups
        // but useful in the context of
        // comparing Raise(x) vs Raise(y) where x != y

        let game_mapper: GameMapper<AuctionPokerAction> = GameMapper::new(None);

        use AuctionPokerAction::*;
        use RelativeSize::*;
        let actions = vec![
            Fold,
            Call,
            Raise(CentiPercent(50)),
            Raise(CentiPercent(100)),
            Raise(CentiPercent(150)),
        ];

        // make sure this test is still valid (all actions map to distinct indices)
        assert_eq!(
            actions.len(),
            actions
                .iter()
                .map(|x| {
                    let index: ActionIndex = x.clone().into();
                    index
                })
                .collect::<HashSet<_>>()
                .len(),
            "Underlying assumptions about the uniqueness and validity \
                   of the actions above have changed! Did you change the bet \
                   sizes or the way Into<ActionIndex> is implemented for them? \
                   If so, change the test above."
        );

        let mapped = game_mapper.map_actions(&actions, 0);

        assert_eq!(
            mapped.iter().collect::<HashSet<_>>(),
            actions.iter().collect::<HashSet<_>>(),
            "The mapped actions should be the same as the original actions"
        );

        // Make sure the test is still valid (certain actions map to equivalent indices)
        let action_group_1 = vec![Fold];
        let action_group_2 = vec![Call];
        let action_group_3 = vec![Raise(CentiPercent(50))];
        let action_group_4 = vec![Raise(CentiPercent(51)), Raise(CentiPercent(52)), Raise(CentiPercent(53))];
        let action_group_5 = vec![
            Raise(CentiPercent(101)),
            Raise(CentiPercent(102)),
            Raise(CentiPercent(103)),
            Raise(CentiPercent(104)),
        ];

        let groups = vec![
            action_group_1.clone(),
            action_group_2.clone(),
            action_group_3.clone(),
            action_group_4.clone(),
            action_group_5.clone(),
        ];

        for group in groups.clone() {
            assert_eq!(
                1,
                group
                    .iter()
                    .map(|x| {
                        let index: ActionIndex = x.clone().into();
                        index
                    })
                    .collect::<HashSet<_>>()
                    .len(),
                "Underlying assumptions about the uniqueness and validity \
                       of the actions above have changed! Did you change the bet \
                       sizes or the way Into<ActionIndex> is implemented for them? \
                       If so, change the test above."
            );
        }
        // Make sure that they map to five distinct indices
        assert_eq!(
            groups.len(),
            groups
                .iter()
                .map(|x| {
                    let index: ActionIndex = x[0].clone().into();
                    index
                })
                .collect::<HashSet<_>>()
                .len(),
            "Underlying assumptions about the uniqueness and validity \
                   of the actions above have changed! Did you change the bet \
                   sizes or the way Into<ActionIndex> is implemented for them? \
                   If so, change the test above."
        );

        // Test is valid, now test the actual behavior
        let mut actions = action_group_1.clone();
        actions.extend(action_group_2.clone());
        actions.extend(action_group_3.clone());
        actions.extend(action_group_4.clone());
        actions.extend(action_group_5.clone());

        let mapped = game_mapper.map_actions(&actions, 0);
        assert_eq!(
            5,
            mapped.len(),
            "There should be 5 distinct action groups after mapping"
        );

        // Because median is ill defined for even groups,
        // can be Raise(102) or Raise(103) for the test, test that
        // it's either
        let possible_mapping_1 = vec![
            Fold,
            Call,
            Raise(CentiPercent(50)),
            Raise(CentiPercent(52)),
            Raise(CentiPercent(102)),
        ];
        let possible_mapping_2 = vec![
            Fold,
            Call,
            Raise(CentiPercent(50)),
            Raise(CentiPercent(52)),
            Raise(CentiPercent(103)),
        ];

        // Convert to sets to make sure that the order doesn't matter
        assert!(
            mapped.iter().collect::<HashSet<_>>()
                == possible_mapping_1.iter().collect::<HashSet<_>>()
                || mapped.iter().collect::<HashSet<_>>()
                    == possible_mapping_2.iter().collect::<HashSet<_>>(),
            "The mapped actions should be one of two possible mappings"
        );
    }
}
