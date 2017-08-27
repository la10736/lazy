#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{LazySync, Producer, VoidContext};
use test::Bencher;

mod benchmarks;

struct P<V, F: FnMut() -> V>(F);

impl<V, F: FnMut() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(&mut self, _context: &VoidContext) -> Self::Output {
        self.0()
    }
}

fn param<V: 'static, F: FnMut() -> V + Send + Sync + 'static>(f: F) -> LazySync<V> {
    LazySync::new(Box::new(P(f)))
}


