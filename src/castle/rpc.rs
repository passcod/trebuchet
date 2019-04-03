use super::{data, Missive};
use crate::client::Kind;
use crate::db::models::App;
use crate::{
    rpc::{app_error, RpcDelegate},
    Bus,
};
use jsonrpc_core::{Metadata, Result as RpcResult};
use jsonrpc_macros::IoDelegate;
use log::info;
use regex::Regex;
use rpc_impl_macro::{rpc, rpc_impl_struct};
use serde_json::json;

#[derive(Clone)]
pub struct Rpc {
    /// Castle bus
    bus: Bus<Missive>,
}

impl Rpc {
    pub fn new(bus: Bus<Missive>) -> Self {
        Self { bus }
    }
}

impl RpcDelegate for Rpc {
    fn to_delegate<M>(self) -> IoDelegate<Self, M>
    where
        M: Metadata,
        Self: Sized + Send + Sync,
    {
        self.to_delegate()
    }
}

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String, kind: Kind, name: String, tags: Vec<String>) {
            info!("received greetings from a {:?} client named \"{}\" with tags: {:?} running {}", kind, name, tags, app);
            self.bus.send_top(Missive::Hello { app, kind, name, tags });
        }

        #[rpc(name = "apps:list")]
        pub fn apps_list(&self, filter: Option<String>) -> RpcResult<Vec<App>> {
            let filter = if let Some(r) = filter {
                Some(Regex::new(&r).map_err(|err| app_error(
                    400,
                    "filter is not a valid regexp",
                    Some(json!(err.to_string())))
                )?)
            } else {
                None
            };

            Ok(if let Missive::AppList(list) = data::request(&self.bus, data::Topic::AppList { filter })? {
                list
            } else {
                Vec::new()
            })
        }

        #[rpc(name = "apps:create")]
        pub fn apps_create(&self, name: String, repo: String, build_script: Option<String>) -> RpcResult<App> {
            if let Missive::App(app) = data::request(&self.bus, data::Topic::CreateApp { name, repo, build_script })? {
                Ok(app)
            } else {
                unreachable!()
            }
        }
    }
}
