#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{Lazy, Producer, VoidContext};

mod contract;

struct P<V, F: FnMut() -> V>(F);

impl<V, F: FnMut() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(&mut self, context: &VoidContext) -> Self::Output {
        self.0()
    }
}

fn param<V: 'static, F: FnMut() -> V + 'static>(f: F) -> Lazy<V> {
    Lazy::new(Box::new(P(f)))
}
