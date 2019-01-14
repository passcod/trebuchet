#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::client::WorkerAgentClient;

fn main() {
    armstrong::init();

    ws::connect("ws://127.0.0.1:1879", |sender| {
        WorkerAgentClient::create(sender)
    }).unwrap();
}
