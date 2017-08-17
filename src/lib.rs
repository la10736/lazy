#![feature(optin_builtin_traits)]

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

use std::cell::{UnsafeCell, RefCell, RefMut};
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

trait LazyDelegate<'local, 'container: 'local> {
    type Output;
    type Producer: Producer<Output=Self::Output> + 'container;
    type Smart: Deref<Target=Field<Self::Producer>> + DerefMut;

    fn get(&'container self) -> &Self::Output {
        let mut field = self.smart();
        if field.value.is_none() {
            field.compute();
        }
        field.get_ref()
    }

    fn smart(&'container self) -> Self::Smart;
}

pub struct LazyCheck<P: Producer>(RefCell<Field<P>>);

impl<'local, 'container: 'local, P: Producer + 'container> LazyDelegate<'local, 'container> for LazyCheck<P> {
    type Output = P::Output;
    type Producer = P;
    type Smart = RefMut<'local, Field<P>>;

    fn smart(&'container self) -> Self::Smart {
        self.0.borrow_mut()
    }
}

impl<P: Producer> LazyCheck<P>
{
    pub fn new(producer: P) -> Self
    {
        LazyCheck(RefCell::new(Field::new(producer)))
    }
}

pub struct LazyCheckParam<P: Producer>
{
    pub lazy: LazyCheck<P>
}

impl<P: Producer> LazyCheckParam<P>
{
    pub fn new(producer: P) -> Self {
        LazyCheckParam { lazy: LazyCheck::new(producer) }
    }
}

pub struct Lazy<P: Producer>(UnsafeCell<Field<P>>);

impl<'local, 'container: 'local, P: Producer + 'container> LazyDelegate<'local, 'container> for Lazy<P> {
    type Output = P::Output;
    type Producer = P;
    type Smart = SmartFieldCell<'local, P>;

    fn smart(&'container self) -> Self::Smart {
        SmartFieldCell(&self.0)
    }
}


impl<P: Producer> Lazy<P>
{
    pub fn new(producer: P) -> Self
    {
        Lazy(UnsafeCell::new(Field::new(producer)))
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

trait SmartContainer<'local, P: Producer, S: Deref<Target=Field<P>> + DerefMut + 'local + ? Sized> {
    fn smart<'container: 'local>(&'container self) -> S;
}

type ThreadSafeSmartContainer<'local, P> = std::sync::MutexGuard<'local, Field<P>>;

impl<'local, P: Producer + 'local> SmartContainer<'local, P, ThreadSafeSmartContainer<'local, P>> for Mutex<Field<P>>
{
    fn smart<'container: 'local>(&'container self) -> ThreadSafeSmartContainer<'local, P> {
        self.lock().unwrap()
    }
}

struct SmartFieldCell<'local, P: Producer + 'local>(&'local UnsafeCell<Field<P>>);

impl<'local, P: Producer + 'local> Deref for SmartFieldCell<'local, P> {
    type Target = Field<P>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0.get()
        }
    }
}

impl<'local, P: Producer + 'local> DerefMut for SmartFieldCell<'local, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0.get()
        }
    }
}

impl<'local, P: Producer + 'local> SmartContainer<'local, P, SmartFieldCell<'local, P>> for UnsafeCell<Field<P>>
{
    fn smart<'container: 'local>(&'container self) -> SmartFieldCell<'local, P> {
        SmartFieldCell(self)
    }
}

trait ValueReference<'local, 'producer: 'local, P: Producer + 'producer> {
    fn get_ref(&'local self) -> &'producer P::Output;
}

impl<'local, 'producer, P, T> ValueReference<'local, 'producer, P> for T
    where 'producer: 'local,
          P: Producer + 'producer,
          T: Deref<Target=Field<P>> + 'local
{
    fn get_ref(&'local self) -> &'producer P::Output {
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
        field.get_ref()
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

    mod lazy_check {
        use super::*;

        fn param<P: Producer>(producer: P) -> LazyCheckParam<P> {
            LazyCheckParam::new(producer)
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
