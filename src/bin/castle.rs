#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use trebuchet::castle::Server;
use trebuchet::central;

fn main() {
    trebuchet::init();
    let (bus, terminal) = central();

    // Larnach Castle postcode
    ws::listen("127.0.0.1:9077", |wstx| Server::new(wstx, bus.clone().launch())).unwrap();
    bus.kill();
    terminal.join().unwrap();
}
