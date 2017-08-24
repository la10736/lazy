#![feature(optin_builtin_traits)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

pub use lazy::Lazy;
//pub use lazy_thread_safe::{LazyThreadSafeParam, ThreadSafeProducer};

mod lazy;
//mod lazy_thread_safe;

pub trait Producer<C> {
    type Output;

    fn produce(&mut self, context: &C) -> Self::Output;
}

impl<V, C, F: FnMut(&C) -> V> Producer<C> for F {
    type Output = V;

    fn produce(&mut self, context: &C) -> V {
        self(context)
    }
}

pub struct VoidContext {}

static VOID_CONTEXT: VoidContext = VoidContext {};

struct Field<C, P: Producer<C>> {
    value: Option<P::Output>,
    producer: Option<P>
}

impl<C, P: Producer<C>> Field<C, P> {
    fn new(producer: P) -> Self
    {
        Field { value: None, producer: Some(producer) }
    }

    fn compute(&mut self, context: &C) {
        if let Some(mut producer) = self.producer.take() {
            self.value = Some(producer.produce(context))
        }
    }
}

trait SmartField<C, P: Producer<C>>: std::ops::Deref<Target=Field<C, P>> + std::ops::DerefMut {}

trait LazyDelegate<'local, 'container: 'local> {
    type Output;
    type Context;
    type Producer: Producer<Self::Context, Output=Self::Output> + 'container;
    type Smart: SmartField<Self::Context, Self::Producer>;

    fn get(&'container self, context: &Self::Context) -> &Self::Output {
        let mut field = self.smart();
        if field.value.is_none() {
            field.compute(context);
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
