use super::*;
use std::sync::{Mutex, Arc};

#[test]
fn should_return_42() {
    let s = param(|| 42);

    assert_eq!(&42, s.get());
}

#[test]
fn should_return_42_also_after_changed_backing_field_value() {
    #[allow(non_upper_case_globals)]
    static mut seed: i32 = 42;

    let s = param(|| unsafe { seed });

    assert_eq!(&42, s.get());

    unsafe { seed = 43 };
    assert_eq!(&42, s.get());
}

#[test]
fn should_call_producer_just_once() {
    // Should be used both for plain and thread safe producer
    let sentinel = Arc::new(Mutex::new(0));

    let s = sentinel.clone();
    let p = param(move || {
        *s.lock().unwrap() += 1;
        42
    });

    assert_eq!(&42, p.get());
    p.get();
    assert_eq!(1, *sentinel.lock().unwrap());
}

#[test]
fn should_work_with_string_too() {
    let p = param(|| "string slice");

    assert_eq!(&"string slice", p.get());
}

#[test]
fn use_producer_trait() {
    struct C { v: i32 }
    struct P {};
    impl Producer<C> for P {
        type Output = i32;

        fn produce(self, context: &C) -> Self::Output { 30 + context.v }
    }

    let producer = P {};
    let context = C { v: 12 };
    let p = LazyProperty::new(producer);

    assert_eq!(&42, p.get(&context));
}

#[test]
fn use_fnonece_producer() {
    let a = "42".to_string();

    let producer = move || a;

    let p = param(producer);

    assert_eq!(&"42", p.get());
}

#[test]
fn use_function_as_producer() {
    fn producer() -> i32 { 42 };

    let p = param(producer);

    assert_eq!(&42, p.get());
}

#[test]
fn lazy_property() {
    struct S {
        name: String,
        property: LazyProperty<String, S>
    }

    impl S {
        fn new(n: &str) -> Self {
            S {
                name: n.to_string(),
                property: LazyProperty::new(|c: &Self| format!("{} 42", c.name)),
            }
        }

        fn get_property(&self) -> &str {
            self.property.get(self)
        }
    }

    let obj = S::new("Fortytwo");

    assert_eq!("Fortytwo 42", obj.get_property())
}

#[test]
fn lazy_property_call_method() {
    struct S {
        name: String,
        property: LazyProperty<String, S>
    }

    impl S {
        fn new(n: &str) -> Self {
            S {
                name: n.to_string(),
                property: LazyProperty::new(Self::resolve_property),
            }
        }

        fn get_property(&self) -> &str {
            self.property.get(self)
        }

        fn resolve_property(&self) -> String {
            format!("{} 42", self.name)
        }
    }

    let obj = S::new("Fortytwo");

    assert_eq!("Fortytwo 42", obj.get_property())
}