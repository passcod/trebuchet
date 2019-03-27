#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use trebuchet::castle;

fn main() {
    let args = castle::arguments().get_matches();
    let (server, bus, terminal) = castle::init(&args);

    // Larnach Castle postcode
    ws::listen(server, |wstx| {
        let bus = bus.clone().launch();
        castle::Server::create(castle::Rpc::new(bus.clone()), wstx, bus)
    })
    .unwrap();
    bus.kill();
    terminal.join().unwrap();
}
