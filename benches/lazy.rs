#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{Lazy, Producer};
use test::Bencher;

mod benchmarks;

fn param<P: Producer>(producer: P) -> Lazy<P> {
    Lazy::new(producer)
}


