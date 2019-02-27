use crate::inflight::Inflight;
use crate::proto::Worker;
use crate::rpc::RpcHandler;
use arc_swap::ArcSwap;
use jsonrpc_core::{IoHandler, Params, Result as RpcResult};
use log::{debug, info};
use rpc_impl_macro::{rpc, rpc_impl_struct};
use rpds::HashTrieMap;
use serde_json::json;
use uuid::Uuid;

pub struct Server {
    /// Own websocket end
    sender: ws::Sender,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

#[derive(Clone, Default)]
pub struct Rpc {
    /// Source of worker data
    state: ArcSwap<State>,
}

#[derive(Clone, Default)]
pub struct State {
    // worker name -> definition
    workers: HashTrieMap<String, Worker>,
}

impl State {
    pub fn update_workers(&self, workers: HashTrieMap<String, Worker>) -> Self {
        Self { workers }
    }
}

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            debug!("received greetings from {}", app);
        }

        #[rpc(name = "core.agent.hello")]
        pub fn agent_hello(&self, id: Option<Uuid>) -> RpcResult<Uuid> {
            Ok(id.unwrap())
        }
    }
}

impl Server {
    pub fn new(sender: ws::Sender) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(Rpc::default().to_delegate());

        Self {
            inflight: Inflight::default(),
            rpc,
            sender,
        }
    }
}

impl RpcHandler for Server {
    const PROTOCOL: &'static str = "armstrong/core";

    fn sender(&self) -> &ws::Sender {
        &self.sender
    }

    fn inflight(&self) -> &Inflight {
        &self.inflight
    }

    fn rpc(&self) -> &IoHandler {
        &self.rpc
    }
}

impl ws::Handler for Server {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        self.rpc_on_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted");
        self.notify(
            "greetings",
            Params::Array(
                json!([format!("ArmstrongCore/{}", env!("CARGO_PKG_VERSION"))])
                    .as_array()
                    .unwrap()
                    .to_owned(),
            ),
            &[],
        )
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
