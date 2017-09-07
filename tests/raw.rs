#![feature(test)]
extern crate lazy_init;
extern crate test;

pub use lazy_init::{Lazy, LazyProperty, Producer, VoidContext};

mod contract;


struct P<V, F: FnMut() -> V>(F);

impl<V, F: FnMut() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(&mut self, _context: &VoidContext) -> Self::Output {
        self.0()
    }
}

fn param<V: 'static, F: FnMut() -> V + 'static>(f: F) -> Lazy<V> {
    Lazy::new(Box::new(P(f)))
}
