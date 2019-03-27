#![forbid(unsafe_code)]
#![deny(bare_trait_objects)]
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
mod error;
mod inflight;
mod message;
pub mod rpc;

pub use bus::{central, Bus};
pub use error::Error as CommonError;

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

    let (own_level, sub_level, backtrace, clean) = match verbosity {
        -128...-4 => ("error", "error", None, true),
        -3 => ("warn", "error", None, true),
        -2 => ("info", "error", None, true),
        -1 => ("info", "warn", None, true),
        0 => ("info", "warn", None, false),
        1 => ("debug", "info", None, false),
        2 => ("trace", "info", None, false),
        3 => ("trace", "debug", Some("1"), false),
        4 => ("trace", "trace", Some("1"), false),
        5...127 => ("trace", "trace", Some("full"), false),
    };

    let init_logger = move || {
        let mut l = env_logger::builder();

        if clean {
            l.default_format_module_path(false);
            l.default_format_timestamp(false);
        } else {
            l.default_format_timestamp_nanos(true);
        }

        l.init();
    };

    let level = format!("{}={},ws={}", env!("CARGO_PKG_NAME"), own_level, sub_level);
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", &level);
        init_logger();
        debug!("set log level to {}", level);
    } else {
        init_logger();
    }

    if env::var("RUST_BACKTRACE").is_err() {
        if let Some(bt) = backtrace {
            debug!("setting backtrace level to {}", bt);
            env::set_var("RUST_BACKTRACE", bt);
        }
    }
}
