use super::*;
use std::cell::{RefCell, RefMut};

struct LazyImpl<C, P: Producer<C>>(RefCell<Field<C, P>>);

impl<'local, 'container: 'local, C: 'container, P: Producer<C> + 'container> LazyDelegate<'local, 'container> for LazyImpl<C, P> {
    type Output = P::Output;
    type Producer = P;
    type Context = C;
    type Smart = RefMut<'local, Field<C, P>>;

    fn smart(&'container self) -> Self::Smart {
        self.0.borrow_mut()
    }
}

impl<'local, C, P: Producer<C>> SmartField<C, P> for RefMut<'local, Field<C, P>> {}

impl<C, P: Producer<C>> LazyImpl<C, P>
{
    fn new(producer: P) -> Self
    {
        LazyImpl(RefCell::new(Field::new(producer)))
    }
}

pub struct LazyValue<V, C>(LazyImpl<C, BoxedProducer<V, C>>);

impl<V> LazyValue<V, VoidContext>
{
    pub fn new(producer: Box<Producer<VoidContext, Output=V>>) -> Self {
        LazyValue(LazyImpl::new(BoxedProducer(producer)))
    }

    pub fn get(&self) -> &V {
        self.0.get(&VOID_CONTEXT)
    }
}

pub type Lazy<V> = LazyValue<V, VoidContext>;

struct BoxedProducer<V, C>(Box<Producer<C, Output=V>>);

impl<V, C> Producer<C> for BoxedProducer<V, C> {
    type Output = V;

    fn produce(&mut self, context: &C) -> Self::Output {
        self.0.produce(context)
    }
}

pub struct LazyProperty<V, C>(LazyImpl<C, BoxedProducer<V, C>>);

impl<V, C> LazyProperty<V, C>
{
    pub fn new(producer: Box<Producer<C, Output=V>>) -> Self {
        LazyProperty(LazyImpl::new(BoxedProducer(producer)))
    }

    pub fn get(&self, context: &C) -> &V {
        self.0.get(context)
    }
}
