use crate::agent::AgentHandle;
use crate::inflight::Inflight;
use crate::proto::Worker;
use crate::rpc::{app_error, RpcHandler};
use jsonrpc_core::{IoHandler, Params, Result as RpcResult};
use log::{debug, info};
use rpc_macro::{rpc, rpc_impl_struct};
use serde_json::json;

pub trait WorkerSource {
    fn register_worker(&self, worker: Worker);
    fn get_worker(&self, name: &str) -> Option<Worker>;
    fn unregister_worker(&self, name: &str);
}

pub struct WorkerServer {
    /// Own websocket end
    sender: ws::Sender,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

pub struct WorkerServerRpc {
    /// Source of worker data
    source: AgentHandle,
}

rpc_impl_struct! {
    impl WorkerServerRpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            debug!("received greetings from {}", app);
        }

        #[rpc(name = "worker.register")]
        pub fn worker_register(&self, worker: Worker) -> RpcResult<bool> {
            self.source.register_worker(worker);
            Ok(true)
        }

        #[rpc(name = "worker.unregister")]
        pub fn worker_unregister(&self, name: String) -> RpcResult<bool> {
            self.source.unregister_worker(&name);
            Ok(true)
        }

        #[rpc(name = "worker.get")]
        pub fn worker_get(&self, name: String) -> RpcResult<Worker> {
            self.source.get_worker(&name).ok_or(app_error(404, "worker not found", None))
        }
    }
}

impl WorkerServer {
    pub fn create(sender: ws::Sender, source: AgentHandle) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(WorkerServerRpc { source }.to_delegate());

        Self {
            inflight: Inflight::default(),
            rpc,
            sender,
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
            Params::Array(
                json!([format!("armstrong agent v{}", env!("CARGO_PKG_VERSION"))])
                    .as_array()
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
