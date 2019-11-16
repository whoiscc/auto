//

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Iterator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DFAutoBuilder<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    graph: HashMap<S, HashMap<T, S>>,
    start_state: S,
    accept_state_set: HashSet<S>,
}

impl<S, T> DFAutoBuilder<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn with_start_state(start_state: S) -> Self {
        Self {
            graph: HashMap::new(),
            start_state,
            accept_state_set: HashSet::new(),
        }
    }
}

impl<S, T> DFAutoBuilder<S, T>
where
    S: Eq + Hash + Clone,
    T: Eq + Hash,
{
    pub fn trans(mut self, from: S, trans: T, to: S) -> Self {
        if !self.graph.contains_key(&from) {
            self.graph.insert(from.clone(), HashMap::new());
        }
        if let Some(old_to) = self.graph.get_mut(&from).unwrap().insert(trans, to.clone()) {
            if old_to != to {
                panic!("duplicated transition");
            }
        }
        self
    }
}

impl<S, T> DFAutoBuilder<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn accept(mut self, state: S) -> Self {
        self.accept_state_set.insert(state);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DFAutoBlueprint<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    graph: HashMap<S, HashMap<T, S>>,
    start_state: S,
    accept_state_set: HashSet<S>,
}

impl<S, T> DFAutoBuilder<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn finalize(self) -> DFAutoBlueprint<S, T> {
        DFAutoBlueprint {
            graph: self.graph,
            start_state: self.start_state,
            accept_state_set: self.accept_state_set,
        }
    }
}

impl<S, T> DFAutoBlueprint<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn start_state(&self) -> &S {
        &self.start_state
    }

    pub fn accept_state_set(&self) -> &HashSet<S> {
        &self.accept_state_set
    }

    pub fn iterate_transitions(&self) -> impl Iterator<Item = (&S, &T, &S)> {
        self.graph
            .iter()
            .flat_map(|(from, trans_to)| trans_to.iter().map(move |(trans, to)| (from, trans, to)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DFAuto<'b, S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    blueprint: &'b DFAutoBlueprint<S, T>,
    current_state: S,
}

impl<S, T> DFAutoBlueprint<S, T>
where
    S: Eq + Hash + Clone,
    T: Eq + Hash,
{
    pub fn create(&self) -> DFAuto<S, T> {
        DFAuto {
            blueprint: self,
            current_state: self.start_state().clone(),
        }
    }
}

impl<'b, S, T> DFAuto<'b, S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn current_state(&self) -> &S {
        &self.current_state
    }

    pub fn is_accepted(&self) -> bool {
        self.blueprint
            .accept_state_set()
            .contains(self.current_state())
    }

    pub fn test_trigger(&self, trans: &T) -> bool {
        self.blueprint
            .graph
            .get(self.current_state())
            .unwrap()
            .contains_key(trans)
    }
}

impl<'b, S, T> DFAuto<'b, S, T>
where
    S: Eq + Hash + Clone,
    T: Eq + Hash,
{
    pub fn trigger(&mut self, trans: &T) {
        self.current_state = self
            .blueprint
            .graph
            .get(self.current_state())
            .unwrap()
            .get(trans)
            .unwrap()
            .clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_auto() {
        let _builder = DFAutoBuilder::with_start_state(0)
            .trans(0, "0 -> 1", 1)
            .accept(1);
    }

    #[test]
    #[should_panic]
    fn build_auto_with_duplicated_trans() {
        let _builder = DFAutoBuilder::with_start_state(0)
            .trans(0, "0 -> 1", 1)
            .trans(0, "0 -> 1", 2);
    }

    #[test]
    fn build_auto_with_redundant_info() {
        let _builder = DFAutoBuilder::with_start_state(0)
            .trans(0, "0 -> 1", 1)
            .trans(0, "0 -> 1", 1)
            .accept(1)
            .accept(1);
    }

    #[test]
    fn blueprint() {
        let dfa = DFAutoBuilder::with_start_state(0)
            .trans(0, "0 -> 1", 1)
            .trans(0, "0 -> 0", 0)
            .trans(1, "1 -> 1", 1)
            .accept(1)
            .finalize();
        assert_eq!(dfa.start_state(), &0);
        assert_eq!(dfa.accept_state_set(), &vec![1].into_iter().collect());
        assert_eq!(dfa.iterate_transitions().count(), 3);
    }

    #[test]
    fn trigger_auto() {
        let dfa = DFAutoBuilder::with_start_state(0)
            .trans(0, "0 -> 1", 1)
            .trans(0, "0 -> 0", 0)
            .trans(1, "1 -> 1", 1)
            .accept(1)
            .finalize();
        let mut auto = dfa.create();
        assert_eq!(auto.current_state(), &0);
        assert!(!auto.is_accepted());
        assert!(auto.test_trigger(&"0 -> 0"));
        assert!(!auto.test_trigger(&"0 -> 2"));
        auto.trigger(&"0 -> 1");
        assert_eq!(auto.current_state(), &1);
        assert!(auto.is_accepted());
    }
}
