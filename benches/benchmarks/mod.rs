use super::*;

#[bench]
fn get_10000(b: &mut Bencher) {
    let p = param(|| 42);

    b.iter(move || {
        let n = test::black_box(10000);
        for _ in 0..n { p.get(); }
    })
}

#[bench]
fn get_100000(b: &mut Bencher) {
    let p = param(|| 42);

    b.iter(move || {
        let n = test::black_box(100000);
        for _ in 0..n { p.get(); }
    })
}
