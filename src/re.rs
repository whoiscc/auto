use crate::nfa::{NFAutoBlueprint, NFAutoBuilder};
use std::hash::Hash;

enum RePriv<T> {
    Plain(T),
    ZeroOrMore(Box<RePriv<T>>),
    Concat(Box<RePriv<T>>, Box<RePriv<T>>),
    Either(Box<RePriv<T>>, Box<RePriv<T>>),
    Wildcard,
}

pub struct Re<T>(RePriv<T>);

impl<T> RePriv<T>
where
    T: Eq + Hash + Clone,
{
    pub fn compile(self) -> NFAutoBlueprint<u64, T> {
        let builder = NFAutoBuilder::with_start_state(0).accept(1);
        let mut counter = 2;
        self.recursive_compile(builder, &mut counter, 0, 1)
            .finalize()
    }

    fn recursive_compile(
        self,
        builder: NFAutoBuilder<u64, T>,
        counter: &mut u64,
        left: u64,
        right: u64,
    ) -> NFAutoBuilder<u64, T> {
        match self {
            RePriv::Plain(trans) => builder.connect(left, trans, right),
            RePriv::ZeroOrMore(inner) => {
                let (inner_left, inner_right) = (*counter, *counter + 1);
                *counter += 2;
                let builder = builder
                    .connect_void(left, inner_left)
                    .connect_void(inner_right, right)
                    .connect_void(inner_right, inner_left)
                    .connect_void(left, right);
                inner.recursive_compile(builder, counter, inner_left, inner_right)
            }
            RePriv::Concat(first, second) => {
                let middle = *counter;
                *counter += 1;
                let builder = first.recursive_compile(builder, counter, left, middle);
                second.recursive_compile(builder, counter, middle, right)
            }
            RePriv::Either(first, second) => {
                let (first_left, first_right, second_left, second_right) =
                    (*counter, *counter + 1, *counter + 2, *counter + 3);
                *counter += 4;
                let builder = builder
                    .connect_void(left, first_left)
                    .connect_void(left, second_left)
                    .connect_void(first_right, right)
                    .connect_void(second_right, right);
                let builder = first.recursive_compile(builder, counter, first_left, first_right);
                second.recursive_compile(builder, counter, second_left, second_right)
            }
            RePriv::Wildcard => builder.connect_wildcard(left, right),
        }
    }
}

impl<T> Re<T> {
    pub fn plain(trans: T) -> Self {
        Self(RePriv::Plain(trans))
    }

    pub fn zero_or_more(inner: Self) -> Self {
        Self(RePriv::ZeroOrMore(Box::new(inner.0)))
    }

    pub fn concat(first: Self, second: Self) -> Self {
        Self(RePriv::Concat(Box::new(first.0), Box::new(second.0)))
    }

    pub fn either(first: Self, second: Self) -> Self {
        Self(RePriv::Either(Box::new(first.0), Box::new(second.0)))
    }

    pub fn wildcard() -> Self {
        Self(RePriv::Wildcard)
    }
}

impl<T> Re<T>
where
    T: Eq + Hash + Clone,
{
    pub fn compile(self) -> NFAutoBlueprint<u64, T> {
        self.0.compile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_and_match() {
        // (a|b)*c
        let bp = Re::concat(
            Re::zero_or_more(Re::either(Re::plain('a'), Re::plain('b'))),
            Re::plain('c'),
        )
        .compile();
        let mut nfa = bp.create();
        for c in "ababbabc".chars() {
            assert!(!nfa.is_accepted());
            nfa.trigger(&c);
            assert!(!nfa.is_dead());
        }
        assert!(nfa.is_accepted());
        nfa.trigger(&'d');
        assert!(nfa.is_dead());
    }
}
