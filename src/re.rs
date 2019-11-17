use crate::nfa::{NFAutoBlueprint, NFAutoBuilder};
use std::convert::Into;
use std::hash::Hash;

pub enum Re<T, R>
where
    R: Into<Re<T, R>>,
{
    Plain(T),
    ZeroOrMore(R),
    Concat(R, R),
    Either(R, R),
}

impl<T, R> Re<T, R>
where
    T: Eq + Hash + Clone,
    R: Into<Re<T, R>>,
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
            Re::Plain(trans) => builder.connect(left, trans, right),
            Re::ZeroOrMore(inner) => {
                let (inner_left, inner_right) = (*counter, *counter + 1);
                *counter += 2;
                let builder = builder
                    .connect_void(left, inner_left)
                    .connect_void(inner_right, right)
                    .connect_void(inner_right, inner_left)
                    .connect_void(left, right);
                inner
                    .into()
                    .recursive_compile(builder, counter, inner_left, inner_right)
            }
            Re::Concat(first, second) => {
                let middle = *counter;
                *counter += 1;
                let builder = first
                    .into()
                    .recursive_compile(builder, counter, left, middle);
                second
                    .into()
                    .recursive_compile(builder, counter, middle, right)
            }
            Re::Either(first, second) => {
                let (first_left, first_right, second_left, second_right) =
                    (*counter, *counter + 1, *counter + 2, *counter + 3);
                *counter += 4;
                let builder = builder
                    .connect_void(left, first_left)
                    .connect_void(left, second_left)
                    .connect_void(first_right, right)
                    .connect_void(second_right, right);
                let builder =
                    first
                        .into()
                        .recursive_compile(builder, counter, first_left, first_right);
                second
                    .into()
                    .recursive_compile(builder, counter, second_left, second_right)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ReBox<T>(std::boxed::Box<Re<T, ReBox<T>>>);

    impl<T> Into<Re<T, ReBox<T>>> for ReBox<T> {
        fn into(self) -> Re<T, ReBox<T>> {
            *self.0
        }
    }

    #[test]
    fn compile() {
        // (a | b)*c
        let re = Re::Concat(
            ReBox(Box::new(Re::ZeroOrMore(ReBox(Box::new(Re::Either(
                ReBox(Box::new(Re::Plain('a'))),
                ReBox(Box::new(Re::Plain('b'))),
            )))))),
            ReBox(Box::new(Re::Plain('c'))),
        );
        let bp = re.compile();
        let mut auto = bp.create();
        for trans in "abababbc".chars() {
            assert!(!auto.is_accepted());
            auto.trigger(&trans);
            assert!(!auto.is_dead());
        }
        assert!(auto.is_accepted());
        auto.trigger(&'d');
        assert!(auto.is_dead());
    }
}
