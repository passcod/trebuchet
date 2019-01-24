#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter, clippy::or_fun_call, clippy::needless_pass_by_value)]

pub mod agent;
pub mod client;
pub mod core;
mod inflight;
mod message;
pub mod proto;
mod rpc;
pub mod system;

pub fn init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "armstrong=info,ws=info");
    }

    env_logger::init();
}
