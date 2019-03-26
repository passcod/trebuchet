use crate::rpc::{RpcClient, RpcRemote};
use clap::{App, Arg, ArgMatches, SubCommand};
use jsonrpc_core::Params;
use log::info;

pub fn arguments<'a, 'b>() -> App<'a, 'b> {
    super::arguments()
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
        .subcommand(
            SubCommand::with_name("releases:list")
                .about("list releases for an app")
                .visible_alias("releases")
                .arg(
                    Arg::with_name("releases:filter")
                        .value_name("FILTER")
                        .help("Regexp filter over the release versions")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("releases:status")
                        .long("status")
                        .value_name("STATUS")
                        .help("Show only releases of a particular status")
                        .takes_value(true)
                        .possible_values(&["ready", "building", "todo"]),
                ),
        )
        .subcommand(SubCommand::with_name("releases:sync").about("sync releases from source repo"))
        .subcommand(SubCommand::with_name("releases:build").about("build a specific release"))
        .subcommand(SubCommand::with_name("releases:rebuild").about("rebuild a release"))
}

pub fn handler(remote: RpcRemote, _args: ArgMatches) {
    let cr = remote.clone();
    remote
        .call("apps:list", Params::Array(Vec::new()), &[], move |res| {
            info!("res: {:?}", res);
            cr.kill(None).expect("failed to kill socket");
        })
        .expect("failed to send command");
}