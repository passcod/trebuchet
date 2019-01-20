#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::agent::Server;

fn main() {
    armstrong::init();

    // Palais Idéal
    ws::listen("127.0.0.1:1879", Server::new).unwrap();
}
