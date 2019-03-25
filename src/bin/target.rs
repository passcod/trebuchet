#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use gethostname::gethostname;
use std::env;
use trebuchet::client::{Client, Kind};

fn main() {
    trebuchet::init();

    let name = env::var("TREBUCHET_NAME")
        .or_else(|_| gethostname().into_string())
        .unwrap_or("anonymous".into());

    let tags: Vec<String> = env::var("TREBUCHET_TAGS")
        .map(|t| t.split_whitespace().map(|s| s.into()).collect())
        .unwrap_or(Vec::new());

    ws::connect("ws://127.0.0.1:9077", |sender| {
        Client::create(sender, Kind::Target, name.clone(), tags.clone())
    })
    .unwrap();
}
