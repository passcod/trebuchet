pub use super::Missive;
use crate::client::Kind;
use crate::db::models::App;
use crate::{rpc::RpcDelegate, Bus};
use jsonrpc_core::{Metadata, Result as RpcResult};
use jsonrpc_macros::IoDelegate;
use log::info;
use rpc_impl_macro::{rpc, rpc_impl_struct};

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
            Ok(Vec::new())
        }
    }
}
