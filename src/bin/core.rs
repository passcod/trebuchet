#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::core::Server;

fn main() {
    armstrong::init();

    // First phone call in New Zealand
    ws::listen("127.0.0.1:1878", Server::new).unwrap();
}
