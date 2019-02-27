use crate::inflight::Inflight;
use crate::proto::Worker;
use crate::rpc::RpcHandler;
use jsonrpc_core::{IoHandler, Params};
use log::info;
use rpc_impl_macro::{rpc, rpc_impl_struct};
use serde_json::json;

/// Client from Worker to Agent.
pub struct WorkerAgentClient {
    sender: ws::Sender,
    inflight: Inflight,
    rpc: IoHandler,
}

pub struct WorkerAgentRpc;

rpc_impl_struct! {
    impl WorkerAgentRpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            info!("received greetings from {}", app);
        }
    }
}

impl WorkerAgentClient {
    pub fn create(sender: ws::Sender) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(WorkerAgentRpc.to_delegate());

        Self {
            sender,
            inflight: Inflight::default(),
            rpc,
        }
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

        let worker = Worker::new("sample", vec![], vec![], vec![]).unwrap();

        self.call(
            "worker.register",
            Params::Map(json!(worker).as_object_mut().unwrap().clone()),
            &[],
            |res| {
                info!("got response from agent: {:?}", res);
            },
        )?;

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
