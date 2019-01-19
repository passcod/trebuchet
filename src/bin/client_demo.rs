#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use armstrong::client::WorkerAgentClient;

fn main() {
    armstrong::init();

    ws::connect("ws://127.0.0.1:1879", |sender| {
        WorkerAgentClient::create(sender)
    })
    .unwrap();
}

use jsonrpc_core::{Error, FutureResult, Result};
use rpc_macro::{rpc, rpc_impl_struct};

struct RpcDerived;
rpc_impl_struct! {
    impl RpcDerived {
        /// Returns a protocol version
        #[rpc(notification, name = "version")]
        pub fn protocol_version(&self) {
            // Ok("version1".into())
        }

        /// Adds two numbers and returns a result
        pub fn add(&self, a: u64, b: u64) -> Result<u64> {
            Ok(a + b)
        }
    }
}
