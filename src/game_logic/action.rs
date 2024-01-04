use crate::constants::*;
use std::fmt::Debug;
use std::hash::Hash;
pub use std::ops::RangeInclusive as StdRange;

pub type ActionIndex = u8;
pub trait Action:
    Clone + Debug + Filterable + Into<ActionIndex> + From<ActionIndex>
{
}

pub type ActionFilter<A> = (Filter<A>, A);

#[derive(Debug, Clone)]
pub struct ActionMapper<A: Filterable> {
    filters: Vec<ActionFilter<A>>,
}

impl<A: Filterable> ActionMapper<A> {
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
    pub fn map(&self, action: A) -> A {
        for (filter, mapped_action) in &self.filters {
            if filter.accepts(&action) {
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

    pub fn map_and_index(&self, action: A, depth: usize, index : ActionIndex) -> (A, ActionIndex) {
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
        // TODO: figure out what to do if functions map to two different groups
        //       right now it's taking a greedy approach
        // TODO: perhaps its a good precondition (checked by debug asserts)
        //       to expect that for some action that is captured by a filter F
        //       it maps to another action from the original legal set,
        //       which is also captured by F, would enforce legality of actions
        //       and consistency of groups implicitly
        // TODO: bruh this doesn't reduce the action space lol
        let mapper = &self.depth_specific_maps[depth];
        match mapper {
            Some(mapper) => actions
                .iter()
                .map(|action| mapper.map(action.clone()))
                .collect(),
            None => actions.clone(),
        }
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
