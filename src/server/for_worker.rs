use crate::inflight::Inflight;
use crate::proto::Worker;
use crate::rpc::{parse_params, RpcDefiner, RpcHandler};
use jsonrpc_core::{IoHandler, Params, Value};
use log::info;
use serde_json::json;

pub trait WorkerSource {
    fn register_worker(&self, worker: Worker);
    fn get_worker(&self, name: &str) -> Option<Worker>;
    fn unregister_worker(&self, name: &str);
}

pub struct WorkerServer<W: WorkerSource + Clone> {
    /// Own websocket end
    sender: ws::Sender,

    /// Source of worker data
    source: W,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

impl<W: WorkerSource + Clone> WorkerServer<W> {
    pub fn create(sender: ws::Sender, source: W) -> Self {
        let mut rpc = IoHandler::new();

        rpc.add_method("worker.register", |params| {
            let worker: Worker = parse_params(params)?;
            source.register_worker(worker);
            Ok(Value::Bool(true))
        });
        rpc.add_method("worker.get", |_| Ok(Value::Bool(true)));
        rpc.add_method("worker.unregister", |_| Ok(Value::Bool(true)));

        Self {
            sender,
            source,
            inflight: Inflight::default(),
            rpc,
        }
    }
}

impl<W: WorkerSource + Clone> RpcHandler for WorkerServer<W> {
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

impl<W: WorkerSource + Clone> RpcDefiner for WorkerServer<W> {
    fn rpc(&mut self) -> &mut IoHandler {
        &mut self.rpc
    }

    fn init_rpc(&mut self) {
        self.define_method("worker.register", |worker: Worker| {
            info!("hello world {:?}", worker);
            // this.source().register_worker(worker);
            Ok(json!(true))
        });
    }
}

impl<W: WorkerSource + Clone> ws::Handler for WorkerServer<W> {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        self.rpc_on_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted for worker");
        self.init_rpc();
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
