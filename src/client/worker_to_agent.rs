use crate::rpc::RpcHandler;
use crate::inflight::Inflight;
use futures::Future;
use jsonrpc_core::{IoHandler, Params};

/// Client from Worker to Agent.
pub struct WorkerAgentClient {
    sender: ws::Sender,
    inflight: Inflight,
    rpc: IoHandler,
}

impl WorkerAgentClient {
    pub fn create(sender: ws::Sender) -> Self {
        let mut rpc = IoHandler::new();

        rpc.add_notification("greetings", |params| {
            info!("received greetings from agent: {:?}", params);
        });

        Self { sender, inflight: Inflight::default(), rpc }
    }
}

impl RpcHandler for WorkerAgentClient {
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

impl ws::Handler for WorkerAgentClient {
    fn build_request(&mut self, url: &url::Url) -> ws::Result<ws::Request> {
        self.rpc_build_request(url)
    }

    fn on_response(&mut self, res: &ws::Response) -> ws::Result<()> {
        self.rpc_on_response(res)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connected to agent");

        self.call("worker.register", Params::Map(vec![
            ("foo".into(), "bar".into())
        ].into_iter().collect()), None)?.inspect(|res| {
            info!("got response from agent: {:?}", res);
        });

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        // info!() something out
    }
}
