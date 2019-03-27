#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use trebuchet::client::{command, init, Client, Kind};

fn main() {
    let args = command::arguments().get_matches();
    let (server, name, tags) = init(&args);

    ws::connect(server, |sender| {
        let args = args.clone();
        Client::create(
            command::Rpc,
            sender,
            Kind::Command,
            name.clone(),
            tags.clone(),
            move |remote| command::handler(remote, args.clone()),
        )
    })
    .expect("failed to start websocket client");
}
