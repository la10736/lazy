#![feature(optin_builtin_traits)]

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

pub use lazy::LazyParam;
pub use lazy_thread_safe::{LazyThreadSafeParam, ThreadSafeProducer};

mod lazy;
mod lazy_thread_safe;

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

trait SmartField<P: Producer>: std::ops::Deref<Target=Field<P>> + std::ops::DerefMut {}

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
