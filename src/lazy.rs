use super::*;
use std::cell::{RefCell, RefMut};

struct LazyImpl<P: Producer>(RefCell<Field<P>>);

impl<'local, 'container: 'local, P: Producer + 'container> LazyDelegate<'local, 'container> for LazyImpl<P> {
    type Output = P::Output;
    type Producer = P;
    type Smart = RefMut<'local, Field<P>>;

    fn smart(&'container self) -> Self::Smart {
        self.0.borrow_mut()
    }
}

impl<'local, P: Producer> SmartField<P> for RefMut<'local, Field<P>> {}

impl<P: Producer> LazyImpl<P>
{
    fn new(producer: P) -> Self
    {
        LazyImpl(RefCell::new(Field::new(producer)))
    }
}

pub struct Lazy<P: Producer>(LazyImpl<P>);

impl<P: Producer> Lazy<P>
{
    pub fn new(producer: P) -> Self {
        Lazy(LazyImpl::new(producer))
    }

    pub fn get(&self) -> &P::Output {
        self.0.get()
    }
}

//pub struct LazyProperty<V, C>(LazyImpl<P>);
//
//impl<P: Producer> Lazy<P>
//{
//    pub fn new(producer: P) -> Self {
//        LazyProperty(LazyImpl::new(producer))
//    }
//
//    pub fn get(&self, ) -> &P::Output {
//        self.0.get()
//    }
//}
