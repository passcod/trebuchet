use crate::inflight::Inflight;
use crate::proto::{Constraint, Worker};
use crate::rpc::{app_error, RpcHandler};
use crate::system::System;
use arc_swap::ArcSwap;
use jsonrpc_core::{IoHandler, Params, Result as RpcResult};
use log::{debug, info};
use rpc_impl_macro::{rpc, rpc_impl_struct};
use rpds::HashTrieMap;
use serde_json::json;
use std::sync::Arc;

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
    system: Arc<System>,

    // worker name -> definition
    workers: HashTrieMap<String, Worker>,
}

impl State {
    pub fn update_workers(&self, workers: HashTrieMap<String, Worker>) -> Self {
        Self {
            system: self.system.clone(),
            workers,
        }
    }
}

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            debug!("received greetings from {}", app);
        }

        #[rpc(name = "agent.checkConstraints")]
        pub fn check_constraints(&self, constraints: Vec<Constraint>) -> RpcResult<Option<usize>> {
            Ok(self.state.lease().system.check_constraints(&constraints))
        }

        #[rpc(name = "worker.register")]
        pub fn worker_register(&self, worker: Worker) -> RpcResult<bool> {
            self.state.rcu(|old| {
                let worker = worker.clone();
                old.update_workers(old.workers.insert(worker.name.clone(), worker))
            });

            Ok(true)
        }

        #[rpc(name = "worker.unregister")]
        pub fn worker_unregister(&self, name: String) -> RpcResult<bool> {
            self.state.rcu(|old| old.update_workers(old.workers.remove(&name)));

            Ok(true)
        }

        #[rpc(name = "worker.get")]
        pub fn worker_get(&self, name: String) -> RpcResult<Worker> {
            (&self.state.lease().workers).clone().get(&name).cloned()
            .ok_or(app_error(404, "worker not found", None))
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

impl ws::Handler for Server {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        self.rpc_on_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted");
        self.notify(
            "greetings",
            Params::Array(
                json!([format!("ArmstrongAgent/{}", env!("CARGO_PKG_VERSION"))])
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
