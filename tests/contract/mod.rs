use super::*;
use std::sync::{Mutex, Arc};
use lazy_init::Producer;

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

    let p = param(|| {
        *sentinel.lock().unwrap() += 1;
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
    struct P {};
    impl Producer for P {
        type Output = i32;

        fn produce(&mut self) -> Self::Output { 42 }
    }

    let producer = P {};
    let p = param(producer);

    assert_eq!(&42, p.get());
}

//#[test]
//fn use_fnonece_producer() {
//    let a = "42".to_string();
//
//    let producer = move || a;
//
//    let p = param(producer);
//
//    assert_eq!(&"42", p.get());
//}

#[test]
fn use_function_as_producer() {
    fn producer() -> i32 { 42 };

    let p = param(producer);

    assert_eq!(&42, p.get());
}