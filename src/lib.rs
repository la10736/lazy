#![feature(optin_builtin_traits)]

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

use std::cell::UnsafeCell;
use std::sync::Mutex;


pub trait Producer {
    type Output;

    fn produce(&self) -> Self::Output;
}

impl<V, F: Fn() -> V> Producer for F {
    type Output = V;

    fn produce(&self) -> V {
        self()
    }
}

struct Field<P: Producer> {
    value: Option<P::Output>,
    producer: P
}

impl<P: Producer> Field<P> {
    fn new(producer: P) -> Self
    {
        Field { value: None, producer: producer }
    }

    fn compute(&mut self) {
        self.value = Some(self.producer.produce())
    }
}

pub struct Lazy<P: Producer>(UnsafeCell<Field<P>>);

impl<P: Producer> Lazy<P>
{
    pub fn new(producer: P) -> Self
    {
        Lazy(UnsafeCell::new(Field::new(producer)))
    }

    pub fn get(&self) -> &P::Output {
        let mut field = self.0.smart();
        if field.value.is_none() {
            field.compute();
        }
        field.extract()
    }
}

pub struct LazyParam<P: Producer>
{
    pub lazy: Lazy<P>
}

impl<P: Producer> LazyParam<P>
{
    pub fn new(producer: P) -> Self {
        LazyParam { lazy: Lazy::new(producer) }
    }
}

pub trait ThreadSafeProducer: Producer + Send + Sync {}

impl<P: Producer + Send + Sync> ThreadSafeProducer for P {}

pub struct LazyThreadSafe<P: ThreadSafeProducer>(Mutex<Field<P>>);

use std::ops::{Deref, DerefMut};

trait SmartContainer<'local, P: Producer, S: Deref<Target=Field<P>> + DerefMut + 'local> {
    fn smart<'container: 'local>(&'container self) -> S;
}

trait ExtractValue<'local, 'producer: 'local, P: Producer + 'producer> {
    fn extract(&'local self) -> &'producer P::Output;
}

impl<'a, P: Producer> SmartContainer<'a, P, std::sync::MutexGuard<'a, Field<P>>> for Mutex<Field<P>>
{
    fn smart<'b: 'a>(&'b self) -> std::sync::MutexGuard<'a, Field<P>> {
        self.lock().unwrap()
    }
}

impl<'a, 'b: 'a, P: Producer + 'b> ExtractValue<'a, 'b, P> for std::sync::MutexGuard<'a, Field<P>>
{
    fn extract(&'a self) -> &'b P::Output {
        unsafe {
            match self.value {
                Some(ref v) => &*(v as *const P::Output),
                None => debug_unreachable!()
            }
        }
    }
}

struct SmartFieldCell<'a, P: Producer + 'a>(&'a UnsafeCell<Field<P>>);

impl<'a, P: Producer + 'a> Deref for SmartFieldCell<'a, P> {
    type Target = Field<P>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0.get()
        }
    }
}

impl<'a, P: Producer + 'a> DerefMut for SmartFieldCell<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0.get()
        }
    }
}

impl<'a, P: Producer + 'a> SmartContainer<'a, P, SmartFieldCell<'a, P>> for UnsafeCell<Field<P>>
{
    fn smart<'c: 'a>(&'c self) -> SmartFieldCell<'a, P> {
        SmartFieldCell(self)
    }
}

impl<'a, 'b: 'a, P: Producer + 'b> ExtractValue<'a, 'b, P> for SmartFieldCell<'a, P> {
    fn extract(&'a self) -> &'b P::Output {
        unsafe {
            match self.value {
                Some(ref v) => &*(v as *const P::Output),
                None => debug_unreachable!()
            }
        }
    }
}

impl<P: ThreadSafeProducer> LazyThreadSafe<P>
    where P: Producer + Send + Sync
{
    pub fn new(producer: P) -> Self
    {
        LazyThreadSafe(Mutex::new(Field::new(producer)))
    }

    pub fn get(&self) -> &P::Output {
        let mut field = self.0.smart();
        if field.value.is_none() {
            field.compute();
        }
        field.extract()
    }
}

pub struct LazyThreadSafeParam<P: ThreadSafeProducer>
{
    pub lazy: LazyThreadSafe<P>
}

impl<P: ThreadSafeProducer> LazyThreadSafeParam<P>
{
    pub fn new(producer: P) -> Self {
        LazyThreadSafeParam { lazy: LazyThreadSafe::new(producer) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod lazy {
        use super::*;

        fn param<P: Producer>(producer: P) -> LazyParam<P> {
            LazyParam::new(producer)
        }

        mod contract;
    }

    mod lazy_thread_safe {
        use super::*;

        fn param<P: ThreadSafeProducer>(producer: P) -> LazyThreadSafeParam<P> {
            LazyThreadSafeParam::new(producer)
        }

        mod contract;

        #[test]
        fn multiple_threads_can_access_to_the_same_property() {
            use std::thread::spawn;

            let s = std::sync::Arc::new(param(|| 42));

            let handles = (0..10).map(|_| {
                let ss = s.clone();
                spawn(move || assert_eq!(&42, ss.lazy.get()))
            }
            ).collect::<Vec<_>>();

            for h in handles {
                h.join().unwrap()
            }
        }
    }
}
