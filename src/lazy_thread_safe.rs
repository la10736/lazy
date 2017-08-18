use super::*;
use std::sync::Mutex;

pub trait ThreadSafeProducer: Producer + Send + Sync {}

impl<P: Producer + Send + Sync> ThreadSafeProducer for P {}

struct LazyThreadSafe<P: ThreadSafeProducer>(Mutex<Field<P>>);

type ThreadSafeSmartContainer<'local, P> = std::sync::MutexGuard<'local, Field<P>>;

impl<'local, 'container: 'local, P: ThreadSafeProducer + 'container> LazyDelegate<'local, 'container> for LazyThreadSafe<P> {
    type Output = P::Output;
    type Producer = P;
    type Smart = ThreadSafeSmartContainer<'local, P>;

    fn smart(&'container self) -> ThreadSafeSmartContainer<'local, P> {
        self.0.lock().unwrap()
    }
}

impl<'local, P: Producer> SmartField<P> for ThreadSafeSmartContainer<'local, P> {}

impl<P: ThreadSafeProducer> LazyThreadSafe<P>
    where P: Producer + Send + Sync
{
    fn new(producer: P) -> Self
    {
        LazyThreadSafe(Mutex::new(Field::new(producer)))
    }
}

pub struct LazyThreadSafeParam<P: ThreadSafeProducer>
{
    lazy: LazyThreadSafe<P>
}

impl<P: ThreadSafeProducer> LazyThreadSafeParam<P>
{
    pub fn new(producer: P) -> Self {
        LazyThreadSafeParam { lazy: LazyThreadSafe::new(producer) }
    }

    pub fn get(&self) -> &P::Output {
        self.lazy.get()
    }
}
