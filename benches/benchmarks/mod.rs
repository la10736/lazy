use super::*;

#[bench]
fn get_1000(b: &mut Bencher) {
    let p = param(|| 42);

    b.iter(|| for _ in 0..1000 { p.get(); })
}

#[bench]
fn get_100000(b: &mut Bencher) {
    let p = param(|| 42);

    b.iter(|| for _ in 0..100000 { p.get(); })
}
