#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use trebuchet::client::{init, target, Client, Kind};

fn main() {
    let args = target::arguments().get_matches();
    let (server, name, tags) = init(&args);

    ws::connect(server, |sender| {
        let args = args.clone();
        Client::create(
            target::Rpc,
            sender,
            Kind::Target,
            name.clone(),
            tags.clone(),
            move |remote| target::handler(remote, args.clone()),
        )
    })
    .expect("failed to start websocket client");
}
