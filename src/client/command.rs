use crate::{
    db::models,
    rpc::{param_list, RpcClient, RpcDelegate, RpcRemote},
};
use clap::{App, Arg, ArgMatches, SubCommand};
use jsonrpc_core::{Metadata, Value};
use jsonrpc_macros::IoDelegate;
use log::{error, info, warn};
use rpc_impl_macro::{rpc, rpc_impl_struct};
use serde_json::{from_value, json};

pub struct Rpc;

impl RpcDelegate for Rpc {
    fn to_delegate<M>(self) -> IoDelegate<Self, M>
    where
        M: Metadata,
        Self: Sized + Send + Sync,
    {
        self.to_delegate()
    }
}

pub fn arguments<'a, 'b>() -> App<'a, 'b> {
    super::arguments("Trebuchet command client")
        .bin_name("trebuchet")
        .subcommand(
            SubCommand::with_name("apps:list")
                .about("list all apps")
                .visible_alias("apps")
                .arg(
                    Arg::with_name("filter")
                        .value_name("FILTER")
                        .help("Regexp filter over the app names")
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("apps:edit").about("reconfigure an app"))
        .subcommand(
            SubCommand::with_name("apps:create")
                .about("configure a new app")
                .arg(
                    Arg::with_name("build_script")
                        .long("build-script")
                        .value_name("SCRIPT")
                        .help("Custom script to build the app")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .help("Name of the app")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("repo")
                        .value_name("REPO")
                        .help("Source git repository. Supports `github:user/repo` shorthand")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("releases:list")
                .about("list releases for an app")
                .visible_alias("releases")
                .arg(
                    Arg::with_name("filter")
                        .value_name("FILTER")
                        .help("Regexp filter over the release versions")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("status")
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

pub fn handler(remote: RpcRemote, args: ArgMatches) {
    let cr = remote.clone();
    let close = move || {
        cr.kill(None).expect("failed to kill socket");
    };

    let r = if let Some(args) = args.subcommand_matches("apps:list") {
        let has_filter = args.value_of("filter").is_some();
        let filter = args
            .value_of("filter")
            .map(|s| Value::String(s.into()))
            .unwrap_or(json!(null));

        remote.call("apps:list", param_list(vec![filter]), move |res| {
            let apps: Vec<models::App> = from_value(res.map_err(|err| {
                close();
                err
            })?)?;

            if apps.is_empty() {
                if has_filter {
                    error!("no apps matched! perhaps check your filter");
                } else {
                    warn!("no apps yet");
                }
            } else {
                info!("showing {} apps:", apps.len());
                for app in &apps {
                    info!("{} ({})", app.name, app.repo);
                }
            }

            close();
            Ok(())
        })
    } else if let Some(args) = args.subcommand_matches("apps:create") {
        let name = Value::String(args.value_of("name").unwrap().into());
        let repo = Value::String(args.value_of("repo").unwrap().into());
        let build_script = args
            .value_of("build_script")
            .map(|s| Value::String(s.into()))
            .unwrap_or(json!(null));

        remote.call(
            "apps:create",
            param_list(vec![name, repo, build_script]),
            move |res| {
                res.map(|_| {
                    info!("done");
                    close();
                })
                .map_err(|err| {
                    close();
                    err.into()
                })
            },
        )
    } else {
        error!("missing command");
        return close();
    };

    r.expect("failed to send command");
}

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            info!("received greetings from {}", app);
        }
    }
}
