#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::or_fun_call,
    clippy::needless_pass_by_value
)]

#[macro_use]
extern crate diesel;

mod bus;
pub mod castle;
pub mod client;
pub mod db;
mod inflight;
mod message;
pub mod rpc;

pub use crate::bus::{central, Bus};

pub fn init() {
    dotenv::dotenv().unwrap_or_else(|err| log::debug!("No .env file loaded: {:?}", err));

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "trebuchet=info,ws=info");
    }

    env_logger::init();
}
