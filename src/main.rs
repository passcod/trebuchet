#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

pub mod proto;

fn main() {
    println!("Hello, world!");
}
