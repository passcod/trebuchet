use crate::bus::{central, Bus};
use clap::ArgMatches;
use log::info;
use std::thread::JoinHandle;

mod args;
mod data;
mod rpc;
mod server;
mod worker;

pub use args::arguments;
pub use rpc::Rpc;
pub use server::Server;
pub use worker::{worker, Missive};

pub fn init(args: &ArgMatches) -> (String, Bus<Missive>, JoinHandle<()>) {
    let verbosity = args.occurrences_of("v") as i8 - args.occurrences_of("q") as i8;
    crate::init_with_level(verbosity);

    let host = args.value_of("host").expect("bad --host option");
    let port = args.value_of("port").expect("bad --port option");
    let server = format!("{}:{}", host, port);

    let (bus, terminal) = central();
    data::data_service(bus.clone());

    info!("Setting up trebuchet on {}", server);
    (server, bus, terminal)
}
