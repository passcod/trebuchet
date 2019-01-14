#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use armstrong::agent::Agent;
use armstrong::server::WorkerServer;

fn main() {
    env_logger::init();
    println!("🌈 Hello, wonderful world!\n");

    // Palais Idéal
    ws::listen("127.0.0.1:1879", |sender| {
        WorkerServer::create(sender, Agent::arced())
    })
    .unwrap();
}
