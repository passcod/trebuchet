use super::{worker, Missive};
use crate::client::Kind;
use crate::inflight::Inflight;
use crate::rpc::RpcHandler;
use crate::Bus;
use jsonrpc_core::{IoHandler, Params, Result as RpcResult};
use log::{debug, info, trace};
use rpc_impl_macro::{rpc, rpc_impl_struct};
use std::thread::{spawn, JoinHandle};
use uuid::Uuid;

pub struct Server {
    /// Own websocket end
    sender: ws::Sender,

    /// Castle bus
    bus: Bus<Missive>,

    /// Handler thread for this connection
    thread: JoinHandle<()>,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

#[derive(Clone)]
pub struct Rpc {
    /// Castle bus
    bus: Bus<Missive>,
}

impl Rpc {
    fn new(bus: Bus<Missive>) -> Self {
        Self { bus }
    }
}

rpc_impl_struct! {
    impl Rpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String, kind: Kind, name: String, tags: Vec<String>) {
            info!("received greetings from a {:?} client named \"{}\" with tags: {:?} running {}", kind, name, tags, app);
            self.bus.send_top(Missive::Hello { app, kind, name, tags });
        }

        #[rpc(name = "core.agent.hello")]
        pub fn agent_hello(&self, id: Option<Uuid>) -> RpcResult<Uuid> {
            Ok(id.unwrap())
        }
    }
}

impl Server {
    pub fn new(sender: ws::Sender, bus: Bus<Missive>) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(Rpc::new(bus.clone()).to_delegate());

        let workws = sender.clone();
        let workbus = bus.clone();
        let thread = spawn(|| worker(workws, workbus));

        Self {
            bus,
            thread,
            inflight: Inflight::default(),
            rpc,
            sender,
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.bus.send_own(Missive::Exit);
    }
}

impl RpcHandler for Server {
    const PROTOCOL: &'static str = "trebuchet/castle";

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
        debug!("connection accepted");
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
