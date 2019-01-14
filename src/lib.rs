#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod agent;
pub mod client;
mod inflight;
mod message;
pub mod proto;
mod rpc;
pub mod server;
pub mod system;

pub fn init() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "armstrong=info,ws=info");
    }

    env_logger::init();
}
