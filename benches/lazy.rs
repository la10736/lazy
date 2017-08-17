#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{LazyParam, Producer};
use test::Bencher;

mod benchmarks;

fn param<P: Producer>(producer: P) -> LazyParam<P> {
    LazyParam::new(producer)
}


