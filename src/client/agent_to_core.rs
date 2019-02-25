use crate::inflight::Inflight;
use crate::rpc::RpcHandler;
use jsonrpc_core::IoHandler;
use log::{debug, info};
use rpc_impl_macro::{rpc, rpc_impl_struct};

/// Client from Agent to Core.
pub struct AgentCoreClient {
    sender: ws::Sender,
    inflight: Inflight,
    rpc: IoHandler,
}

pub struct AgentCoreRpc;

rpc_impl_struct! {
    impl AgentCoreRpc {
        #[rpc(notification)]
        pub fn greetings(&self, app: String) {
            debug!("received greetings from {}", app);
        }
    }
}

impl AgentCoreClient {
    pub fn create(sender: ws::Sender) -> Self {
        let mut rpc = IoHandler::new();
        rpc.extend_with(AgentCoreRpc.to_delegate());

        Self {
            sender,
            inflight: Inflight::default(),
            rpc,
        }
    }
}

impl RpcHandler for AgentCoreClient {
    const PROTOCOL: &'static str = "armstrong/agent";

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

impl ws::Handler for AgentCoreClient {
    fn build_request(&mut self, url: &url::Url) -> ws::Result<ws::Request> {
        self.rpc_build_request(url)
    }

    fn on_response(&mut self, res: &ws::Response) -> ws::Result<()> {
        self.rpc_on_response(res)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connected to core");
        // send greeting
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.rpc_on_message(msg)
    }

    fn on_shutdown(&mut self) {
        self.rpc_on_shutdown()
    }
}
