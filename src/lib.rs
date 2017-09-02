#![feature(optin_builtin_traits)]
#![feature(unboxed_closures)]
#![feature(test)]

extern crate test;

mod raw;
mod sync;

pub use raw::{Lazy, LazyProperty};
pub use sync::Lazy as LazySync;
pub use sync::LazyProperty as LazyPropertySync;
pub use sync::SharedProducer;

pub struct VoidContext {}

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

        fn produce(&mut self, _context: &VoidContext) -> Self::Output {
            self.0
        }
    }

    struct Fake(std::cell::UnsafeCell<Field<VoidContext, FakeProducer>>);

    impl Fake {
        fn new(v: i32) -> Self {
            Fake(std::cell::UnsafeCell::new(Field { value: Some(v), producer: None }))
        }
    }

    impl<'local, 'container: 'local> LazyDelegate<'local, 'container> for Fake {
        type Output = i32;
        type Context = VoidContext;
        type Producer = FakeProducer;
        type Smart = &'container mut Field<VoidContext, FakeProducer>;

        fn smart(&'container self) -> Self::Smart {
            unsafe { &mut *self.0.get() }
        }
    }

    impl ! Sync for Fake {}

    #[bench]
    fn get_fake_10000(b: &mut Bencher) {
        let p = Fake::new(42);

        b.iter(move || for _ in 0..10000 { assert_eq!(&42, p.get(&VOID_CONTEXT)) })
    }

    #[bench]
    fn smart_fake_10000(b: &mut Bencher) {
        let p = Fake::new(42);

        b.iter(|| for _ in 0..10000 { if p.smart().value.is_none() { panic!("Should be some") }; })
    }

    #[bench]
    fn fill_fake_10000(b: &mut Bencher) {
        let p = Fake::new(42);
        p.get(&VOID_CONTEXT);
        let mut s = p.smart();

        b.iter(|| for _ in 0..10000 { Fake::fill(&mut s, &VOID_CONTEXT) })
    }

    #[bench]
    fn reference_fake_10000(b: &mut Bencher) {
        let p = Fake::new(42);
        p.get(&VOID_CONTEXT);
        let s = p.smart();

        b.iter(|| for _ in 0..10000 {
            assert_eq!(unsafe { p.extract_reference(&s) }, &42)
        })
    }

    struct RefCellFieldWrap(RefCell<Field<VoidContext, FakeProducer>>);

    impl<'local, C, P: Producer<C>> SmartField<C, P> for RefMut<'local, Field<C, P>> {}

    impl<'local, 'container: 'local> LazyDelegate<'local, 'container> for RefCellFieldWrap {
        type Output = i32;
        type Context = VoidContext;
        type Producer = FakeProducer;
        type Smart = RefMut<'local, Field<Self::Context, Self::Producer>>;

        fn smart(&'container self) -> Self::Smart {
            self.0.borrow_mut()
        }
    }

    #[bench]
    fn get_ref_cell_10000(b: &mut Bencher) {
        let p = RefCellFieldWrap(RefCell::new(Field::new(FakeProducer(42))));

        b.iter(|| for _ in 0..10000 { assert_eq!(&42, p.get(&VOID_CONTEXT)) })
    }

    #[bench]
    fn smart_ref_cell_10000(b: &mut Bencher) {
        let p = RefCellFieldWrap(RefCell::new(Field::new(FakeProducer(42))));
        p.get(&VOID_CONTEXT);

        b.iter(|| for _ in 0..10000 { if p.smart().value.is_none() { panic!("Should be some") }; })
    }

    #[bench]
    fn fill_ref_cell_10000(b: &mut Bencher) {
        let p = RefCellFieldWrap(RefCell::new(Field::new(FakeProducer(42))));
        p.get(&VOID_CONTEXT);
        let mut s = p.smart();

        b.iter(|| for _ in 0..10000 { RefCellFieldWrap::fill(&mut s, &VOID_CONTEXT) })
    }

    #[bench]
    fn reference_ref_cell_10000(b: &mut Bencher) {
        let p = RefCellFieldWrap(RefCell::new(Field::new(FakeProducer(42))));
        p.get(&VOID_CONTEXT);
        let s = p.smart();

        b.iter(|| for _ in 0..10000 {
            assert_eq!(unsafe { p.extract_reference(&s) }, &42)
        })
    }

    #[bench]
    fn dereference_int_pointer_10000(b: &mut Bencher) {
        let fortytwo = 42;
        let ptr = &fortytwo as *const i32;

        b.iter(|| for _ in 0..10000 {
            assert_eq!(unsafe { &*ptr }, &42)
        })
    }

    struct BoxedProducer<V, C>(Box<Producer<C, Output=V>>);

    impl<V, C> Producer<C> for BoxedProducer<V, C> {
        type Output = V;

        fn produce(&mut self, context: &C) -> Self::Output {
            self.0.produce(context)
        }
    }

    struct BoxedWrap(RefCell<Field<VoidContext, BoxedProducer<i32, VoidContext>>>);

    impl BoxedWrap {
        fn new(v: i32) -> Self {
            BoxedWrap(RefCell::new(Field::new(BoxedProducer(Box::new(FakeProducer(v))))))
        }
    }

    impl<'local, 'container: 'local> LazyDelegate<'local, 'container> for BoxedWrap {
        type Output = i32;
        type Context = VoidContext;
        type Producer = BoxedProducer<Self::Output, Self::Context>;
        type Smart = RefMut<'local, Field<Self::Context, Self::Producer>>;

        fn smart(&'container self) -> Self::Smart {
            self.0.borrow_mut()
        }
    }

    #[bench]
    fn get_boxed_10000(b: &mut Bencher) {
        let p = BoxedWrap::new(42);

        b.iter(|| for _ in 0..10000 { assert_eq!(&42, p.get(&VOID_CONTEXT)) })
    }

    #[bench]
    fn smart_boxed_10000(b: &mut Bencher) {
        let p = BoxedWrap::new(42);
        p.get(&VOID_CONTEXT);

        b.iter(|| for _ in 0..10000 { if p.smart().value.is_none() { panic!("Should be some") }; })
    }

    #[bench]
    fn fill_boxed_10000(b: &mut Bencher) {
        let p = BoxedWrap::new(42);
        p.get(&VOID_CONTEXT);
        let mut s = p.smart();

        b.iter(|| for _ in 0..10000 { BoxedWrap::fill(&mut s, &VOID_CONTEXT) })
    }

    #[bench]
    fn reference_boxed_10000(b: &mut Bencher) {
        let p = BoxedWrap::new(42);
        p.get(&VOID_CONTEXT);
        let s = p.smart();

        b.iter(|| for _ in 0..10000 {
            assert_eq!(unsafe { p.extract_reference(&s) }, &42)
        })
    }

    struct LazyValue(BoxedWrap);

    impl LazyValue
    {
        pub fn new(v: i32) -> Self {
            LazyValue(BoxedWrap::new(v))
        }

        pub fn get(&self) -> &i32 {
            self.0.get(&VOID_CONTEXT)
        }
    }

    #[bench]
    fn get_wrapped_boxed_10000(b: &mut Bencher) {
        let l = LazyValue::new(42);

        b.iter(|| for _ in 0..10000 { assert_eq!(&42, l.get()); })
    }
}