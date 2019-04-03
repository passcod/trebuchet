use super::{worker, Missive};
use crate::inflight::Inflight;
use crate::rpc::{RpcClient, RpcDelegate, RpcHandler};
use crate::Bus;
use jsonrpc_core::IoHandler;
use log::debug;
use std::thread;

pub struct Server {
    /// Own websocket end
    sender: ws::Sender,

    /// Castle bus
    bus: Bus<Missive>,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

impl Server {
    pub fn create<R>(rpcd: R, sender: ws::Sender, bus: Bus<Missive>) -> Self
    where
        R: RpcDelegate + Send + Sync + 'static,
    {
        let mut rpc = IoHandler::new();
        rpc.extend_with(rpcd.to_delegate());

        let workws = sender.clone();
        let workbus = bus.clone();
        thread::Builder::new()
            .name(format!("worker thread {}", bus.id))
            .spawn(move || {
                debug!("worker thread start {}", workbus.id);
                worker(workws, workbus.clone());
                debug!("worker thread end {}", workbus.id);
            })
            .expect("failed to start worker thread");

        Self {
            bus,
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

impl RpcClient for Server {
    fn sender(&self) -> ws::Sender {
        self.sender.clone()
    }

    fn inflight(&self) -> Inflight {
        self.inflight.clone()
    }
}

impl RpcHandler for Server {
    const PROTOCOL: &'static str = "trebuchet/castle";

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
