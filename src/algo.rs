use crate::dfa::{DFAutoBlueprint, DFAutoBuilder};
use crate::nfa::NFAutoBlueprint;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

pub fn determinize<S, T>(nfa: &NFAutoBlueprint<S, T>) -> DFAutoBlueprint<BTreeSet<S>, T>
where
    S: Hash + Eq + Ord + Clone,
    T: Hash + Eq + Clone,
{
    let start_state_set: BTreeSet<_> =
        extend_state_set(nfa, &vec![nfa.start_state().clone()].into_iter().collect())
            .into_iter()
            .collect();
    let mut builder = DFAutoBuilder::start(start_state_set.clone());
    let mut unresolved_state_set_list = vec![start_state_set];
    let mut resolved_state_set_set: HashSet<BTreeSet<_>> = HashSet::new();
    while let Some(state_set) = unresolved_state_set_list.pop() {
        let mut aggregated_connections: HashMap<_, HashSet<_>> = HashMap::new();
        let mut aggregated_wildcard_connections = HashSet::new();
        for state in state_set.iter() {
            if nfa.accept_state_set().contains(&state) {
                builder = builder.accept(state_set.clone());
            }

            let connections = nfa.connections_from(state);
            let (option_trans_to_set, option_wildcard_to_set) =
                (connections.plain, connections.wildcard);
            if let Some(trans_to_set) = option_trans_to_set {
                for (trans, to_set) in trans_to_set.iter() {
                    if !aggregated_connections.contains_key(trans) {
                        aggregated_connections.insert(trans.clone(), HashSet::new());
                    }
                    aggregated_connections
                        .get_mut(trans)
                        .unwrap()
                        .extend(extend_state_set(nfa, to_set));
                }
            }
            if let Some(wildcard_to_set) = option_wildcard_to_set {
                aggregated_wildcard_connections.extend(extend_state_set(nfa, wildcard_to_set));
            }
        }
        for (trans, to_hashset) in aggregated_connections {
            let to_btreeset: BTreeSet<_> = to_hashset.clone().into_iter().collect();
            builder = builder.connect(state_set.clone(), trans, to_btreeset.clone());
            if !resolved_state_set_set.contains(&to_btreeset) {
                unresolved_state_set_list.push(to_btreeset);
            }
        }
        if !aggregated_wildcard_connections.is_empty() {
            let wildcard_to: BTreeSet<_> = aggregated_wildcard_connections.into_iter().collect();
            builder = builder.connect_fallback(state_set.clone(), wildcard_to.clone());
            if !resolved_state_set_set.contains(&wildcard_to) {
                unresolved_state_set_list.push(wildcard_to);
            }
        }
        resolved_state_set_set.insert(state_set);
    }
    builder.finalize()
}

pub(crate) fn extend_state_set<S, T>(
    nfa: &NFAutoBlueprint<S, T>,
    state_set: &HashSet<S>,
) -> HashSet<S>
where
    S: Hash + Eq + Clone,
    T: Hash + Eq,
{
    let mut state_set = state_set.clone();
    let fallback = HashSet::new();
    loop {
        let void_reachable: HashSet<_> = state_set
            .iter()
            .flat_map(|state| nfa.connections_from(state).void.unwrap_or(&fallback))
            .cloned()
            .collect();
        let extended = &state_set | &void_reachable;
        if extended.len() == state_set.len() {
            return state_set;
        }
        state_set = extended;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto::Auto;
    use crate::re::Re;

    #[test]
    fn correct_auto() {
        // (a|b)*.(c|d)
        let auto = determinize(
            &Re::concat(
                Re::zero_or_more(Re::either(Re::plain('a'), Re::plain('b'))),
                Re::concat(Re::wildcard(), Re::either(Re::plain('c'), Re::plain('d'))),
            )
            .compile(),
        );
        assert!(auto.create().test("abababb&c".chars()));
        assert!(auto.create().test("ababbba?d".chars()));
        assert!(!auto.create().test("ababbbe-d".chars()));
    }
}
