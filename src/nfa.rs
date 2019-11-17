use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Iterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFAutoBuilder<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    graph: HashMap<S, HashMap<T, HashSet<S>>>,
    void_graph: HashMap<S, HashSet<S>>,
    start_state: S,
    accept_state_set: HashSet<S>,
}

impl<S, T> NFAutoBuilder<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    pub fn with_start_state(start_state: S) -> Self {
        Self {
            graph: HashMap::new(),
            void_graph: HashMap::new(),
            start_state,
            accept_state_set: HashSet::new(),
        }
    }

    pub fn accept(mut self, state: S) -> Self {
        self.accept_state_set.insert(state);
        self
    }
}

impl<S, T> NFAutoBuilder<S, T>
where
    S: Hash + Eq + Clone,
    T: Hash + Eq + Clone,
{
    pub fn connect(mut self, from: S, trans: T, to: S) -> Self {
        if !self.graph.contains_key(&from) {
            self.graph.insert(from.clone(), HashMap::new());
        }
        let trans_to = self.graph.get_mut(&from).unwrap();
        if !trans_to.contains_key(&trans) {
            trans_to.insert(trans.clone(), HashSet::new());
        }
        trans_to.get_mut(&trans).unwrap().insert(to);
        self
    }

    pub fn connect_void(mut self, from: S, to: S) -> Self {
        if !self.void_graph.contains_key(&from) {
            self.void_graph.insert(from.clone(), HashSet::new());
        }
        self.void_graph.get_mut(&from).unwrap().insert(to);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFAutoBlueprint<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    graph: HashMap<S, HashMap<T, HashSet<S>>>,
    void_graph: HashMap<S, HashSet<S>>,
    start_state: S,
    accept_state_set: HashSet<S>,
}

impl<S, T> NFAutoBuilder<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    pub fn finalize(self) -> NFAutoBlueprint<S, T> {
        NFAutoBlueprint {
            graph: self.graph,
            void_graph: self.void_graph,
            start_state: self.start_state,
            accept_state_set: self.accept_state_set,
        }
    }
}

impl<S, T> NFAutoBlueprint<S, T>
where
    S: Hash + Eq,
    T: Hash + Eq,
{
    pub fn start_state(&self) -> &S {
        &self.start_state
    }

    pub fn accept_state_set(&self) -> &HashSet<S> {
        &self.accept_state_set
    }

    pub fn iterate_connections(&self) -> impl Iterator<Item = (&S, Option<&T>, &S)> {
        Iterator::chain(
            self.graph.iter().flat_map(|(from, trans_to)| {
                trans_to.iter().flat_map(move |(trans, to_set)| {
                    to_set.iter().map(move |to| (from, Some(trans), to))
                })
            }),
            self.void_graph
                .iter()
                .flat_map(|(from, to_set)| to_set.iter().map(move |to| (from, None, to))),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFAuto<'b, S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    blueprint: &'b NFAutoBlueprint<S, T>,
    current_state_set: HashSet<S>,
}

impl<S, T> NFAutoBlueprint<S, T>
where
    S: Hash + Eq + Clone,
    T: Hash + Eq,
{
    pub fn create(&self) -> NFAuto<S, T> {
        let mut auto = NFAuto {
            blueprint: self,
            current_state_set: vec![self.start_state().clone()].into_iter().collect(),
        };
        auto.extend_current_state_set();
        auto
    }
}

impl<'b, S, T> NFAuto<'b, S, T>
where
    S: Hash + Eq + Clone,
    T: Hash + Eq,
{
    fn extend_current_state_set(&mut self) {
        let fallback = HashSet::new();
        loop {
            let void_reachable: HashSet<_> = self
                .current_state_set()
                .iter()
                .flat_map(|state| self.blueprint.void_graph.get(state).unwrap_or(&fallback))
                .cloned()
                .collect();
            let extended = self.current_state_set() | &void_reachable;
            if extended.len() == self.current_state_set().len() {
                break;
            }
            self.current_state_set = extended;
        }
    }

    pub fn is_accepted(&self) -> bool {
        !(&self.current_state_set & self.blueprint.accept_state_set()).is_empty()
    }

    pub fn is_dead(&self) -> bool {
        self.current_state_set().is_empty()
    }

    pub fn current_state_set(&self) -> &HashSet<S> {
        &self.current_state_set
    }

    pub fn trigger(&mut self, trans: &T) {
        let placeholder_state = HashMap::new();
        let placeholder_trans = HashSet::new();
        self.current_state_set = self
            .current_state_set()
            .iter()
            .flat_map(|state| {
                self.blueprint
                    .graph
                    .get(state)
                    .unwrap_or(&placeholder_state)
                    .get(trans)
                    .unwrap_or(&placeholder_trans)
            })
            .cloned()
            .collect();
        self.extend_current_state_set();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_nfa() {
        // ab*a
        let bp = NFAutoBuilder::with_start_state(0)
            .connect(0, 'a', 1)
            .connect_void(1, 2)
            .connect(2, 'b', 3)
            .connect_void(3, 4)
            .connect_void(3, 2)
            .connect_void(1, 4)
            .connect(4, 'a', 5)
            .accept(5)
            .finalize();
        let mut auto = bp.create();
        assert_eq!(auto.current_state_set(), &vec![0].into_iter().collect());
        assert!(!auto.is_dead());
        assert!(!auto.is_accepted());
        auto.trigger(&'a');
        assert_eq!(
            auto.current_state_set(),
            &vec![1, 2, 4].into_iter().collect()
        );
        assert!(!auto.is_accepted());
        auto.trigger(&'b');
        assert_eq!(
            auto.current_state_set(),
            &vec![2, 3, 4].into_iter().collect()
        );
        auto.trigger(&'b');
        assert_eq!(
            auto.current_state_set(),
            &vec![2, 3, 4].into_iter().collect()
        );
        auto.trigger(&'a');
        assert_eq!(auto.current_state_set(), &vec![5].into_iter().collect());
        assert!(auto.is_accepted());
        auto.trigger(&'c');
        assert!(auto.is_dead());
    }
}
