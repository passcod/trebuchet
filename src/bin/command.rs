#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use clap::{App, Arg, SubCommand};
use gethostname::gethostname;
use jsonrpc_core::Params;
use std::env;
use trebuchet::client::{Client, Kind};
use trebuchet::rpc::RpcClient;

fn main() {
    trebuchet::init();

    let name = env::var("TREBUCHET_NAME")
        .or_else(|_| gethostname().into_string())
        .unwrap_or("anonymous".into());

    let args = App::new("Trebuchet command client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Configure, monitor, and deploy apps with Trebuchet")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOSTNAME")
                .help("Sets the Trebuchet server to connect to")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets the Trebuchet server port")
                .takes_value(true)
                .default_value("9077"),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .value_name("NAME")
                .help("Identifies to the server with a name")
                .takes_value(true)
                .default_value(&name),
        )
        .arg(
            Arg::with_name("tags")
                .long("tags")
                .value_name("TAGS")
                .help("Identifies to the server with tags (comma-separated)")
                .value_delimiter(","),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("apps:list")
                .about("list all apps")
                .visible_alias("apps")
                .arg(
                    Arg::with_name("apps:filter")
                        .value_name("FILTER")
                        .help("Regexp filter over the app names")
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("apps:edit").about("reconfigure an app"))
        .subcommand(SubCommand::with_name("apps:create").about("configure a new app"))
        .subcommand(SubCommand::with_name("releases").about("list releases for an app"))
        .get_matches();

    let host = args.value_of("host").expect("bad --host option");
    let port = args.value_of("port").expect("bad --port option");

    let name: String = args.value_of("name").expect("bad --name option").into();
    let tags: Vec<String> = args
        .values_of("tags")
        .map(|ts| ts.map(|s| s.to_string()).collect())
        .unwrap_or(Vec::new());

    ws::connect(format!("ws://{}:{}", host, port), |sender| {
        Client::create(sender, Kind::Command, name.clone(), tags.clone()).with_thread(|remote| {
            remote
                .call("apps:list", Params::Array(Vec::new()), &[], |res| {
                    println!("res: {:?}", res);
                })
                .expect("failed to send command");
        })
    })
    .unwrap();
}
