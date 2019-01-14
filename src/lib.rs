#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::stutter)]
#![allow(clippy::non_ascii_literal)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod agent;
mod client;
mod inflight;
mod message;
pub mod proto;
mod server;
pub mod system;
