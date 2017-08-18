#![feature(optin_builtin_traits)]

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

use std::cell::{RefCell, RefMut};
use std::sync::Mutex;
use std::ops::{Deref, DerefMut};


pub trait Producer {
    type Output;

    fn produce(self) -> Self::Output;
}

impl<V, F: FnOnce() -> V> Producer for F {
    type Output = V;

    fn produce(self) -> V {
        self()
    }
}

struct Field<P: Producer> {
    value: Option<P::Output>,
    producer: Option<P>
}

impl<P: Producer> Field<P> {
    fn new(producer: P) -> Self
    {
        Field { value: None, producer: Some(producer) }
    }

    fn compute(&mut self) {
        if let Some(producer) = self.producer.take() {
            self.value = Some(producer.produce())
        }
    }
}

trait SmartField<P: Producer>: Deref<Target=Field<P>> + DerefMut {}

trait LazyDelegate<'local, 'container: 'local> {
    type Output;
    type Producer: Producer<Output=Self::Output> + 'container;
    type Smart: SmartField<Self::Producer>;

    fn get(&'container self) -> &Self::Output {
        let mut field = self.smart();
        if field.value.is_none() {
            field.compute();
        }
        unsafe {
            match field.value {
                Some(ref v) => &*(v as *const Self::Output),
                None => debug_unreachable!()
            }
        }
    }

    fn smart(&'container self) -> Self::Smart;
}

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
