use crate::agent::AgentHandle;
use crate::inflight::Inflight;
use crate::proto::Worker;
use crate::rpc::RpcHandler;
use jsonrpc_core::{IoHandler, Params, Result as RpcResult, Value};
use log::info;
use rpc_macro::rpc_impl_struct;
use serde_json::json;

pub trait WorkerSource {
    fn register_worker(&self, worker: Worker);
    fn get_worker(&self, name: &str) -> Option<Worker>;
    fn unregister_worker(&self, name: &str);
}

pub struct WorkerServer {
    /// Own websocket end
    sender: ws::Sender,

    /// Source of worker data
    source: AgentHandle,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

impl WorkerServer {
    pub fn create(sender: ws::Sender, source: AgentHandle) -> Self {
        Self {
            sender,
            source,
            inflight: Inflight::default(),
            rpc: IoHandler::new(),
        }
    }
}

rpc_impl_struct! {
    impl WorkerServer {
        fn worker_register(&self, worker: Worker) -> RpcResult<bool> {
            self.source.register_worker(worker);
            Ok(true)
        }
    }
}

impl RpcHandler for WorkerServer {
    const PROTOCOL: &'static str = "armstrong/worker";

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

impl ws::Handler for WorkerServer {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        self.rpc_on_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted for worker");
        // delegate here
        self.notify(
            "greetings",
            Params::Map(
                json!({
                    "app": "armstrong agent",
                    "version": env!("CARGO_PKG_VERSION")
                })
                .as_object()
                .unwrap()
                .to_owned(),
            ),
            None,
        )
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
