#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{LazySync, Producer, VoidContext};
use test::Bencher;

mod benchmarks;

struct P<V, F: FnOnce() -> V>(F);

impl<V, F: FnOnce() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(self, _context: &VoidContext) -> Self::Output {
        self.0()
    }
}

fn param<V: 'static, F: FnOnce() -> V + Send + Sync + 'static>(f: F) -> LazySync<V> {
    LazySync::new(Box::new(P(f)))
}


