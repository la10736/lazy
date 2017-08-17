use super::*;

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
fn should_work_with_string_too() {
    let p = param(|| "string slice");

    assert_eq!(&"string slice", p.get());
}
