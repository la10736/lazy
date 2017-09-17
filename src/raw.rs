use super::*;
use std::cell::UnsafeCell;

struct LazyImpl<V, C>(UnsafeCell<Field<C, BoxedProducer<V, C>>>);

impl<'local, 'container: 'local, V: 'container, C: 'container> LazyDelegate<'local, 'container> for LazyImpl<V, C> {
    type Output = V;
    type Context = C;
    type Producer = BoxedProducer<V, C>;
    type Smart = &'container mut Field<C, Self::Producer>;

    fn smart(&'container self) -> Self::Smart {
        unsafe { &mut *self.0.get() }
    }
}

impl<'a, C, P: Producer<C>> SmartField<C, P> for &'a mut Field<C, P> {}

impl<V, C> LazyImpl<V, C>
{
    fn new(producer: BoxedProducer<V, C>) -> Self
    {
        LazyImpl(UnsafeCell::new(Field::new(producer)))
    }
}

pub struct LazyValue<V, C>(LazyImpl<V, C>);

impl<V> LazyValue<V, VoidContext>
{
    pub fn new(producer: Box<ProducerBox<VoidContext, Output=V>>) -> Self {
        LazyValue(LazyImpl::new(BoxedProducer(producer)))
    }

    pub fn get(&self) -> &V {
        self.0.get(&VOID_CONTEXT)
    }
}

pub type Lazy<V> = LazyValue<V, VoidContext>;

struct BoxedProducer<V, C>(Box<ProducerBox<C, Output=V>>);

impl<V, C> Producer<C> for BoxedProducer<V, C> {
    type Output = V;

    fn produce(self, context: &C) -> Self::Output {
        self.0.produce(context)
    }
}

pub struct LazyProperty<V, C>(LazyImpl<V, C>);

impl<V, C> LazyProperty<V, C>
{
    pub fn new<P: Producer<C, Output=V> + 'static>(p: P) -> Self {
        LazyProperty(LazyImpl::new(BoxedProducer(Box::new(p))))
    }

    pub fn get(&self, context: &C) -> &V {
        self.0.get(context)
    }
}
