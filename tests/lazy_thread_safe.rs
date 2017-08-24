#![feature(test)]
extern crate lazy_init;
extern crate test;

//use lazy_init::{LazyThreadSafeParam, ThreadSafeProducer};
//
//mod contract;
//
//fn param<P: ThreadSafeProducer>(producer: P) -> LazyThreadSafeParam<P> {
//    LazyThreadSafeParam::new(producer)
//}
//
//#[test]
//fn multiple_threads_can_access_to_the_same_property() {
//    use std::thread::spawn;
//
//    let s = std::sync::Arc::new(param(|| 42));
//
//    let handles = (0..10).map(|_| {
//        let ss = s.clone();
//        spawn(move || assert_eq!(&42, ss.get()))
//    }
//    ).collect::<Vec<_>>();
//
//    for h in handles {
//        h.join().unwrap()
//    }
//}