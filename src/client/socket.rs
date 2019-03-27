use super::Kind;
use crate::inflight::Inflight;
use crate::rpc::{RpcClient, RpcDelegate, RpcHandler, RpcRemote};
use jsonrpc_core::{IoHandler, Params};
use log::{debug, error, info};
use serde_json::json;
use std::thread::spawn;

/// Client from Worker to Agent.
pub struct Client<F>
where
    F: FnMut(RpcRemote) + Send + 'static,
{
    sender: ws::Sender,
    inflight: Inflight,
    rpc: IoHandler,
    kind: Kind,
    name: String,
    tags: Vec<String>,
    thread: Option<Box<F>>,
}

impl<F: FnMut(RpcRemote) + Send + 'static> Client<F> {
    pub fn create<R>(
        rpcd: R,
        sender: ws::Sender,
        kind: Kind,
        name: String,
        tags: Vec<String>,
        thread: F,
    ) -> Self
    where
        R: RpcDelegate + Send + Sync + 'static,
    {
        let mut rpc = IoHandler::new();
        rpc.extend_with(rpcd.to_delegate());

        Self {
            sender,
            inflight: Inflight::default(),
            rpc,
            kind,
            name,
            tags,
            thread: Some(Box::new(thread)),
        }
    }
}

impl<F: FnMut(RpcRemote) + Send + 'static> RpcClient for Client<F> {
    fn sender(&self) -> ws::Sender {
        self.sender.clone()
    }

    fn inflight(&self) -> Inflight {
        self.inflight.clone()
    }
}

impl<F: FnMut(RpcRemote) + Send + 'static> RpcHandler for Client<F> {
    const PROTOCOL: &'static str = "trebuchet/castle";

    fn rpc(&self) -> &IoHandler {
        &self.rpc
    }
}

impl<F: FnMut(RpcRemote) + Send + 'static> ws::Handler for Client<F> {
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

        let remote = self.remote();
        let mut body = None;
        std::mem::swap(&mut self.thread, &mut body);
        spawn(move || {
            debug!("client body thread start");
            if let Some(mut body) = body {
                body(remote);
            } else {
                error!("client body swap failed");
            }
            debug!("client body thread end");
        });

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
