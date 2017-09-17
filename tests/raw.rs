#![feature(test)]
extern crate lazy_init;

pub use lazy_init::{Lazy, LazyProperty, Producer, VoidContext};

mod contract;


struct P<V, F: FnOnce() -> V>(F);

impl<V, F: FnOnce() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(self, _context: &VoidContext) -> Self::Output {
        self.0()
    }
}

fn param<V: 'static, F: FnOnce() -> V + 'static>(f: F) -> Lazy<V> {
    Lazy::new(Box::new(P(f)))
}
