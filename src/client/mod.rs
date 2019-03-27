use clap::ArgMatches;
use serde_derive::{Deserialize, Serialize};

mod args;
pub mod command;
mod socket;
pub mod target;

pub use args::arguments;
pub use socket::Client;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Kind {
    /// Client that can be deployed to
    Target,

    /// Client that controls ops
    Command,
}

pub fn init(args: &ArgMatches) -> (String, String, Vec<String>) {
    let verbosity = args.occurrences_of("v") as i8 - args.occurrences_of("q") as i8;
    crate::init_with_level(verbosity);

    let host = args.value_of("host").expect("bad --host option");
    let port = args.value_of("port").expect("bad --port option");
    let server = format!("ws://{}:{}", host, port);

    let name: String = args.value_of("name").expect("bad --name option").into();
    let tags: Vec<String> = args
        .values_of("tags")
        .map(|ts| ts.map(|s| s.to_string()).collect())
        .unwrap_or(Vec::new());

    (server, name, tags)
}
