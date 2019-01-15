#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::agent::AgentHandle;
use armstrong::server::WorkerServer;

fn main() {
    armstrong::init();

    // Palais Idéal
    ws::listen("127.0.0.1:1879", |sender| {
        WorkerServer::create(sender, AgentHandle::default())
    })
    .unwrap();
}
