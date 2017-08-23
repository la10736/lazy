#![feature(test)]
extern crate lazy_init;
extern crate test;

use lazy_init::{Lazy, Producer};

mod contract;

fn param<P: Producer>(producer: P) -> Lazy<P> {
    Lazy::new(producer)
}
