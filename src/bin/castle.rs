#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use trebuchet::castle;
use trebuchet::central;

fn main() {
    trebuchet::init();
    let (bus, terminal) = central();

    castle::data_service(bus.clone());

    // Larnach Castle postcode
    ws::listen("127.0.0.1:9077", |wstx| {
        castle::Server::create(castle::Rpc::new(bus.clone()), wstx, bus.clone().launch())
    })
    .unwrap();
    bus.kill();
    terminal.join().unwrap();
}
