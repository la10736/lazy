use super::*;
use std::sync::Mutex;

pub trait SharedProducer<C>: Producer<C> + Send + Sync {}

impl<C, P: Producer<C> + Send + Sync> SharedProducer<C> for P {}

struct BoxedProducer<V, C>(Box<SharedProducer<C, Output=V>>);

impl<V, C> Producer<C> for BoxedProducer<V, C> {
    type Output = V;

    fn produce(&mut self, context: &C) -> Self::Output {
        self.0.produce(context)
    }
}


struct LazySyncImpl<V, C>(Mutex<Field<C, BoxedProducer<V, C>>>);

type SharedSmartContainer<'local, V, C> = std::sync::MutexGuard<'local, Field<C, BoxedProducer<V, C>>>;

impl<'local, 'container: 'local, V: 'container, C: 'container> LazyDelegate<'local, 'container> for LazySyncImpl<V, C> {
    type Output = V;
    type Context = C;
    type Producer = BoxedProducer<V, C>;
    type Smart = SharedSmartContainer<'local, Self::Output, Self::Context>;

    fn smart(&'container self) -> Self::Smart {
        self.0.lock().unwrap()
    }
}

impl<'local, V, C> SmartField<C, BoxedProducer<V, C>> for SharedSmartContainer<'local, V, C> {}

impl<V, C> LazySyncImpl<V, C>
{
    fn new(producer: BoxedProducer<V, C>) -> Self
    {
        LazySyncImpl(Mutex::new(Field::new(producer)))
    }
}

pub struct LazyValue<V, C>(LazySyncImpl<V, C>);

impl<V> LazyValue<V, VoidContext>
{
    pub fn new(producer: Box<SharedProducer<VoidContext, Output=V>>) -> Self {
        LazyValue(LazySyncImpl::new(BoxedProducer(producer)))
    }

    pub fn get(&self) -> &V {
        self.0.get(&VOID_CONTEXT)
    }
}

pub type Lazy<V> = LazyValue<V, VoidContext>;

pub struct LazyProperty<V, C>(LazySyncImpl<V, C>);

impl<V, C> LazyProperty<V, C>
{
    pub fn new<P: SharedProducer<C, Output=V> + 'static>(p: P) -> Self {
        LazyProperty(LazySyncImpl::new(BoxedProducer(Box::new(p) as Box<SharedProducer<C, Output=V>>)))
    }

    pub fn get(&self, context: &C) -> &V {
        self.0.get(context)
    }
}
