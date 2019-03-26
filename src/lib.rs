#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::or_fun_call,
    clippy::needless_pass_by_value
)]

#[macro_use]
extern crate diesel;

use log::debug;
use std::env;

mod bus;
pub mod castle;
pub mod client;
pub mod db;
mod inflight;
mod message;
pub mod rpc;

pub use crate::bus::{central, Bus};

lazy_static::lazy_static! {
    pub static ref HOSTNAME: String = {
        gethostname::gethostname()
            .into_string()
            .unwrap_or("anonymous".into())
    };
}

pub fn init() {
    init_with_level(0)
}

pub fn init_with_level(verbosity: i8) {
    dotenv::dotenv().unwrap_or_else(|err| log::debug!("No .env file loaded: {:?}", err));

    let (own_level, sub_level, backtrace) = match verbosity {
        -128...-3 => ("error", "error", None),
        -2 => ("warn", "error", None),
        -1 => ("info", "error", None),
        0 => ("info", "warn", None),
        1 => ("debug", "info", None),
        2 => ("trace", "info", None),
        3 => ("trace", "debug", Some("1")),
        4 => ("trace", "trace", Some("1")),
        5...127 => ("trace", "trace", Some("full")),
    };

    let level = format!("{}={},ws={}", env!("CARGO_PKG_NAME"), own_level, sub_level);
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", &level);
        env_logger::init();
        debug!("set log level to {}", level);
    } else {
        env_logger::init();
    }

    if env::var("RUST_BACKTRACE").is_err() {
        if let Some(bt) = backtrace {
            debug!("setting backtrace level to {}", bt);
            env::set_var("RUST_BACKTRACE", bt);
        }
    }
}
