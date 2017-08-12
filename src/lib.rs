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

struct Field<'a, V> {
    value: Option<V>,
    producer: Box<Producer<Output=V> + 'a>
}

impl<'a, V> Field<'a, V> {
    fn new<P: Producer<Output=V> + 'a>(producer: P) -> Self
    {
        Field { value: None, producer: Box::new(producer) }
    }

    fn compute(&mut self) {
        self.value = Some(self.producer.produce())
    }
}

pub struct Lazy<'a, P>(UnsafeCell<Field<'a, P>>);

impl<'a, P> Lazy<'a, P>
{
    pub fn new<F: Producer<Output=P> + 'a>(f: F) -> Self
    {
        Lazy(UnsafeCell::new(Field::new(f)))
    }

    pub fn get(&self) -> &P {
        unsafe {
            let field = &mut *self.0.get();
            if field.value.is_none() {
                field.compute();
            }
            match field.value {
                Some(ref v) => v,
                None => debug_unreachable!()
            }
        }
    }
}

pub struct LazyParam<'a, P>
{
    pub lazy: Lazy<'a, P>
}

impl<'a, P> LazyParam<'a, P>
{
    pub fn new<F: Producer<Output=P> + 'a>(f: F) -> Self {
        LazyParam { lazy: Lazy::new(f) }
    }
}

pub struct LazyThreadSafe<'a, P>
{
    field: Mutex<Option<P>>,
    producer: Box<Producer<Output=P> + 'a + Send + Sync>
}

impl<'a, P> LazyThreadSafe<'a, P>
{
    pub fn new<F: Producer<Output=P> + 'a + Send + Sync>(f: F) -> Self
    {
        LazyThreadSafe { field: Mutex::new(None), producer: Box::new(f) }
    }

    pub fn get(&self) -> &P {
        let mut inner = self.field.lock().unwrap();
        if inner.is_none() {
            *inner = Some(self.producer.produce());
        }
        unsafe {
            match *inner {
                Some(ref v) => &*(v as *const P),
                None => debug_unreachable!()
            }
        }
    }
}

pub struct LazyThreadSafeParam<'a, P>
{
    pub lazy: LazyThreadSafe<'a, P>
}

impl<'a, P> LazyThreadSafeParam<'a, P>
{
    pub fn new<F: Producer<Output=P> + 'a + Send + Sync>(f: F) -> Self {
        LazyThreadSafeParam { lazy: LazyThreadSafe::new(f) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod lazy {
        use super::*;

        fn param<'a, P, F: Producer<Output=P> + 'a>(f: F) -> LazyParam<'a, P> {
            LazyParam::new(f)
        }

        mod contract;
    }

    mod lazy_thread_safe {
        use super::*;

        fn param<'a, P, F: Producer<Output=P> + 'a + Send + Sync>(f: F) -> LazyThreadSafeParam<'a, P> {
            LazyThreadSafeParam::new(f)
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
