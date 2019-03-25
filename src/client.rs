use crate::inflight::Inflight;
use crate::rpc::{RpcClient, RpcHandler, RpcRemote};
use jsonrpc_core::{IoHandler, Params};
use log::info;
use rpc_impl_macro::{rpc, rpc_impl_struct};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::thread::spawn;

/// Client from Worker to Agent.
pub struct Client {
    sender: ws::Sender,
    inflight: Inflight,
    rpc: IoHandler,
    kind: Kind,
    name: String,
    tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Kind {
    /// Client that can be deployed to
    Target,

    /// Client that controls ops
    Command,
}

pub struct Rpc;

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            info!("received greetings from {}", app);
        }
    }
}

impl Client {
    pub fn create(sender: ws::Sender, kind: Kind, name: String, tags: Vec<String>) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(Rpc.to_delegate());

        Self {
            sender,
            inflight: Inflight::default(),
            rpc,
            kind,
            name,
            tags,
        }
    }

    pub fn with_thread<F>(self, body: F) -> Self
    where
        F: FnMut(RpcRemote) + Send + 'static,
    {
        let remote = self.remote();
        let mut body = Box::new(body);
        spawn(move || body(remote));

        self
    }
}

impl RpcClient for Client {
    fn sender(&self) -> ws::Sender {
        self.sender.clone()
    }

    fn inflight(&self) -> Inflight {
        self.inflight.clone()
    }
}

impl RpcHandler for Client {
    const PROTOCOL: &'static str = "trebuchet/castle";

    fn rpc(&self) -> &IoHandler {
        &self.rpc
    }
}

impl ws::Handler for Client {
    fn build_request(&mut self, url: &url::Url) -> ws::Result<ws::Request> {
        self.rpc_build_request(url)
    }

    fn on_response(&mut self, res: &ws::Response) -> ws::Result<()> {
        self.rpc_on_response(res)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connected to castle");

        self.notify(
            "greetings",
            Params::Array(
                json!([
                    format!("Trebuchet/{}", env!("CARGO_PKG_VERSION")),
                    &self.kind,
                    &self.name,
                    &self.tags,
                ])
                .as_array()
                .unwrap()
                .clone(),
            ),
            &[],
        )?;

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
