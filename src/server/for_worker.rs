use crate::client::{Client, MessagePassthru};
use crate::inflight::Inflight;
use crate::message::{parse_binary, parse_plain, Rpc};
use crate::proto::Worker;
use futures::Future;
use jsonrpc_core::{IoHandler, Value};
use std::sync::{Arc, RwLock};

pub trait WorkerSource {
    fn register_worker(&mut self, worker: Worker);
    fn get_worker(&self, name: &str) -> Option<&Worker>;
    fn unregister_worker(&mut self, name: &str);
}

pub struct WorkerServer<W: WorkerSource> {
    /// Own websocket end
    sender: ws::Sender,

    /// Source of worker data
    source: Arc<RwLock<W>>,

    /// Pass messages along the core connection
    corepass: MessagePassthru,

    /// Requests currently awaiting response
    inflight: Inflight,

    /// JSON-RPC server handlers
    rpc: IoHandler,
}

impl<W: WorkerSource> WorkerServer<W> {
    fn create(sender: ws::Sender, source: Arc<RwLock<W>>, corepass: MessagePassthru) -> Self {
        let mut rpc = IoHandler::new();

        rpc.add_method("worker.register", |_| Ok(Value::Bool(true)));

        Self {
            sender,
            source,
            corepass,
            inflight: Inflight::default(),
            rpc,
        }
    }
}

impl<W: WorkerSource> Client for WorkerServer<W> {
    fn sender(&self) -> &ws::Sender {
        &self.sender
    }

    fn inflight(&self) -> &Inflight {
        &self.inflight
    }
}

impl<W: WorkerSource> ws::Handler for WorkerServer<W> {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        if req.protocols()?.contains(&&"armstrong/worker") {
            let mut res = ws::Response::from_request(req)?;
            res.set_protocol("armstrong/worker");
            Ok(res)
        } else {
            Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted for worker");
        // send greeting
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        // handle server messages
        if let Some(rpc) = match msg {
            ws::Message::Text(string) => {
                trace!("string message received: {:?}", string);
                parse_plain(&string)
            }
            ws::Message::Binary(raw) => {
                trace!("raw message received: {:?}", raw);
                parse_binary(&raw)
            }
        } {
            match rpc {
                Rpc::Request(req) => {
                    if let Some(res) = self.rpc.handle_rpc_request(req).wait().unwrap() {
                        self.sender.send(json!(res).to_string())?
                    }
                }
                Rpc::Response(res) => self.handle_response(res)?,
            };
        }
        Ok(())
    }

    fn on_shutdown(&mut self) {
        // info!() something out
    }
}
