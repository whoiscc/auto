use std::iter::Iterator;

pub trait Auto {
    type Trans: 'static;

    fn trigger(&mut self, trans: &Self::Trans);

    fn test_trigger(&self, trans: &Self::Trans) -> bool;

    fn is_accepted(&self) -> bool;

    fn test<'t, I>(&mut self, iter: I) -> bool
    where
        I: Iterator<Item = &'t Self::Trans>,
    {
        for trans in iter {
            if !self.test_trigger(trans) {
                return false;
            }
            self.trigger(trans);
        }
        self.is_accepted()
    }
}
