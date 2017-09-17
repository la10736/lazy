#![feature(test)]
extern crate lazy_init;

pub use lazy_init::{LazySync as Lazy, LazyPropertySync as LazyProperty,
                    Producer, VoidContext};

mod contract;

fn param<V: 'static, F: FnOnce() -> V + Send + Sync + 'static>(f: F) -> Lazy<V> {
    Lazy::new(Box::new(P(f)))
}

struct P<V, F: FnOnce() -> V>(F);

impl<V, F: FnOnce() -> V> Producer<VoidContext> for P<V, F> {
    type Output = V;

    fn produce(self, _context: &VoidContext) -> Self::Output {
        self.0()
    }
}

#[test]
fn multiple_threads_can_access_to_the_same_property() {
    use std::thread::spawn;

    let s = std::sync::Arc::new(param(|| 42));

    let handles = (0..10).map(|_| {
        let ss = s.clone();
        spawn(move || assert_eq!(&42, ss.get()))
    }
    ).collect::<Vec<_>>();

    for h in handles {
        h.join().unwrap()
    }
}