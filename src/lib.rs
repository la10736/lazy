#![feature(optin_builtin_traits)]
#![feature(unboxed_closures)]
#![feature(test)]
#![feature(fn_traits)]

#[cfg(test)]
extern crate test;

mod raw;
mod sync;

pub use raw::{Lazy, LazyProperty};
pub use sync::{Lazy as LazySync, LazyProperty as LazyPropertySync, SharedProducer};

pub struct VoidContext {}

pub trait Producer<C> {
    type Output;

    fn produce(self, context: &C) -> Self::Output;
}

pub trait ProducerBox<C> {
    type Output;

    fn produce_box(self: Box<Self>, context: &C) -> Self::Output;
}

impl<'a, C, V> Producer<C> for Box<ProducerBox<C, Output=V> + 'a> {
    type Output = V;

    fn produce(self, context: &C) -> Self::Output {
        self.produce_box(context)
    }
}

impl<'a, C, V> Producer<C> for Box<ProducerBox<C, Output=V> + Send + 'a> {
    type Output = V;

    fn produce(self, context: &C) -> Self::Output {
        self.produce_box(context)
    }
}

impl<C, P> ProducerBox<C> for P
    where P: Producer<C>
{
    type Output = P::Output;

    fn produce_box(self: Box<Self>, context: &C) -> Self::Output {
        self.produce(context)
    }
}

impl<C, V, F> Producer<C> for F
    where F: FnOnce(&C) -> V
{
    type Output = V;

    fn produce(self, context: &C) -> Self::Output {
        self.call_once((context, ))
    }
}

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
        if let Some(producer) = self.producer.take() {
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
        Self::fill(&mut field, context);
        unsafe { self.extract_reference(&field) }
    }

    unsafe fn extract_reference(&'container self, field: &Self::Smart) -> &Self::Output {
        &*(field.value
            .as_ref()
            .expect("Should call fill() before!")
            as *const Self::Output
        )
    }

    fn fill(field: &mut Self::Smart, context: &Self::Context) {
        if field.value.is_none() {
            field.compute(context);
        }
    }

    fn smart(&'container self) -> Self::Smart;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::cell::{RefMut, RefCell};

    struct FakeProducer(i32);

    impl Producer<VoidContext> for FakeProducer {
        type Output = i32;

        fn produce(self, _context: &VoidContext) -> Self::Output {
            self.0
        }
    }

    mod unsafe_cell_wrapper {
        use super::*;

        struct UnsafeCellW(std::cell::UnsafeCell<Field<VoidContext, FakeProducer>>);

        impl UnsafeCellW {
            fn new(v: i32) -> Self {
                UnsafeCellW(std::cell::UnsafeCell::new(Field { value: Some(v), producer: None }))
            }
        }

        impl<'local, 'container: 'local> LazyDelegate<'local, 'container> for UnsafeCellW {
            type Output = i32;
            type Context = VoidContext;
            type Producer = FakeProducer;
            type Smart = &'container mut Field<VoidContext, FakeProducer>;

            fn smart(&'container self) -> Self::Smart {
                unsafe { &mut *self.0.get() }
            }
        }

        impl ! Sync for UnsafeCellW {}

        #[bench]
        fn get(b: &mut Bencher) {
            let p = UnsafeCellW::new(42);

            b.iter(move || {
                let n = test::black_box(10000);
                for _ in 0..n { p.get(&VOID_CONTEXT); }
            })
        }

        #[bench]
        fn smart(b: &mut Bencher) {
            let p = UnsafeCellW::new(42);

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n { p.smart().value.is_none(); }
            })
        }

        #[bench]
        fn fill(b: &mut Bencher) {
            let p = UnsafeCellW::new(42);
            p.get(&VOID_CONTEXT);
            let mut s = p.smart();

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n { UnsafeCellW::fill(&mut s, &VOID_CONTEXT) }
            })
        }

        #[bench]
        fn reference(b: &mut Bencher) {
            let p = UnsafeCellW::new(42);
            p.get(&VOID_CONTEXT);
            let s = p.smart();

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n {
                    unsafe { p.extract_reference(&s) };
                }
            })
        }
    }

    mod ref_cell_wrapper {
        use super::*;

        struct RefCellW(RefCell<Field<VoidContext, FakeProducer>>);

        impl<'local, C, P: Producer<C>> SmartField<C, P> for RefMut<'local, Field<C, P>> {}

        impl<'local, 'container: 'local> LazyDelegate<'local, 'container> for RefCellW {
            type Output = i32;
            type Context = VoidContext;
            type Producer = FakeProducer;
            type Smart = RefMut<'local, Field<Self::Context, Self::Producer>>;

            fn smart(&'container self) -> Self::Smart {
                self.0.borrow_mut()
            }
        }

        #[bench]
        fn get(b: &mut Bencher) {
            let p = RefCellW(RefCell::new(Field::new(FakeProducer(42))));

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n { p.get(&VOID_CONTEXT); }
            })
        }

        #[bench]
        fn smart(b: &mut Bencher) {
            let p = RefCellW(RefCell::new(Field::new(FakeProducer(42))));
            p.get(&VOID_CONTEXT);

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n { p.smart().value.is_none(); }
            })
        }

        #[bench]
        fn fill(b: &mut Bencher) {
            let p = RefCellW(RefCell::new(Field::new(FakeProducer(42))));
            p.get(&VOID_CONTEXT);
            let mut s = p.smart();

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n { RefCellW::fill(&mut s, &VOID_CONTEXT) }
            })
        }

        #[bench]
        fn reference(b: &mut Bencher) {
            let p = RefCellW(RefCell::new(Field::new(FakeProducer(42))));
            p.get(&VOID_CONTEXT);
            let s = p.smart();

            b.iter(|| {
                let n = test::black_box(10000);
                for _ in 0..n {
                    unsafe { p.extract_reference(&s) };
                }
            })
        }
    }

    #[bench]
    fn dereference_int_pointer_10000(b: &mut Bencher) {
        let fortytwo = 42;
        let ptr = &fortytwo as *const i32;

        b.iter(|| {
            let n = test::black_box(10000);
            for _ in 0..n {
                unsafe { &*ptr };
            }
        })
    }
}