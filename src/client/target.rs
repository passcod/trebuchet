use crate::rpc::{RpcDelegate, RpcRemote};
use clap::{App, ArgMatches};
use jsonrpc_core::Metadata;
use jsonrpc_macros::IoDelegate;
use log::info;
use rpc_impl_macro::{rpc, rpc_impl_struct};

pub fn arguments<'a, 'b>() -> App<'a, 'b> {
    super::arguments("Trebuchet target client").bin_name("trebuchet-target")
}

pub fn handler(_remote: RpcRemote, _args: ArgMatches) {}

pub struct Rpc;

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            info!("received greetings from {}", app);
        }
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
