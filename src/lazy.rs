use super::*;
use std::cell::{RefCell, RefMut};

struct Lazy<P: Producer>(RefCell<Field<P>>);

impl<'local, 'container: 'local, P: Producer + 'container> LazyDelegate<'local, 'container> for Lazy<P> {
    type Output = P::Output;
    type Producer = P;
    type Smart = RefMut<'local, Field<P>>;

    fn smart(&'container self) -> Self::Smart {
        self.0.borrow_mut()
    }
}

impl<'local, P: Producer> SmartField<P> for RefMut<'local, Field<P>> {}

impl<P: Producer> Lazy<P>
{
    fn new(producer: P) -> Self
    {
        Lazy(RefCell::new(Field::new(producer)))
    }
}

pub struct LazyParam<P: Producer>
{
    lazy: Lazy<P>
}

impl<P: Producer> LazyParam<P>
{
    pub fn new(producer: P) -> Self {
        LazyParam { lazy: Lazy::new(producer) }
    }

    pub fn get(&self) -> &P::Output {
        self.lazy.get()
    }
}
