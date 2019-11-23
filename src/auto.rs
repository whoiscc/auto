use std::borrow::Borrow;
use std::iter::Iterator;

pub trait Auto {
    type Trans;

    fn trigger(&mut self, trans: &Self::Trans);

    fn test_trigger(&self, trans: &Self::Trans) -> bool;

    fn is_accepted(&self) -> bool;

    fn test<I>(&mut self, iter: I) -> bool
    where
        I: Iterator,
        I::Item: Borrow<Self::Trans>,
    {
        for trans in iter {
            if !self.test_trigger(trans.borrow()) {
                return false;
            }
            self.trigger(trans.borrow());
        }
        self.is_accepted()
    }

    fn search<I>(&mut self, iter: I) -> bool
    where
        I: Iterator,
        I::Item: Borrow<Self::Trans>,
    {
        let mut accepted = false;
        for trans in iter {
            if self.is_accepted() {
                accepted = true;
            }
            if !self.test_trigger(trans.borrow()) {
                return accepted;
            }
            self.trigger(trans.borrow());
        }
        accepted || self.is_accepted()
    }
}
