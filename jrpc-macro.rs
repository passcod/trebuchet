use jsonrpc_core::futures::future::{self, FutureResult};
use jsonrpc_core::{Error, IoHandler, Result};

pub trait Rpc: Sized + Send + Sync + 'static {
    /// Returns a protocol version
    fn protocol_version(&self) -> Result<String>;

    /// Adds two numbers and returns a result
    fn add(&self, _: u64, _: u64) -> Result<u64>;

    /// Performs asynchronous operation
    fn call(&self, _: u64) -> FutureResult<String, Error>;

    /// Transform this into an `IoDelegate`, automatically wrapping
    /// the parameters.
    fn to_delegate<M: jsonrpc_core::Metadata>(self) -> IoDelegate<Self, M> {
        let mut del = IoDelegate::new(self.into());
        del.add_method("protocolVersion", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::protocol_version as fn(&_) -> Result<String>),
                base,
                params,
            )
        });
        del.add_method("add", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::add as fn(&_, u64, u64) -> Result<u64>),
                base,
                params,
            )
        });
        del.add_method("callAsync", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::call as fn(&_, u64) -> FutureResult<String, Error>),
                base,
                params,
            )
        });
        del
    }
}
struct RpcImpl;
impl Rpc for RpcImpl {
    fn protocol_version(&self) -> Result<String> {
        Ok("version1".into())
    }
    fn add(&self, a: u64, b: u64) -> Result<u64> {
        Ok(a + b)
    }
    fn call(&self, _: u64) -> FutureResult<String, Error> {
        future::ok("OK".to_owned())
    }
}
fn main() {
    let mut io = IoHandler::new();
    let rpc = RpcImpl;
    io.extend_with(rpc.to_delegate())
}


struct RpcDerived;

rpc_impl_struct! {
    impl RpcDerived {
        /// Returns a protocol version
        #[rpc(name = "version")]
        fn protocol_version(&self) -> Result<String> {
            Ok("version1".into())
        }

        /// Adds two numbers and returns a result
        fn add(&self, a: u64, b: u64) -> Result<u64> {
            Ok(a + b)
        }

        /// Performs asynchronous operation
        fn call(&self, _: u64) -> FutureResult<String, Error> {
            future::ok("OK".to_owned())
        }
    }
}

impl Rpc for RpcImpl {
    fn protocol_version(&self) -> Result<String> {
        Ok("version1".into())
    }
    fn add(&self, a: u64, b: u64) -> Result<u64> {
        Ok(a + b)
    }
    fn call(&self, _: u64) -> FutureResult<String, Error> {
        future::ok("OK".to_owned())
    }

    fn to_delegate<M: jsonrpc_core::Metadata>(self) -> IoDelegate<Self, M> {
        let mut del = IoDelegate::new(self.into());
        del.add_method("version", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::protocol_version as fn(&_) -> Result<String>),
                base,
                params,
            )
        });
        del.add_method("add", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::add as fn(&_, u64, u64) -> Result<u64>),
                base,
                params,
            )
        });
        del.add_method("callAsync", move |base, params| {
            WrapAsync::wrap_rpc(
                &(Self::call as fn(&_, u64) -> FutureResult<String, Error>),
                base,
                params,
            )
        });
        del
    }
}
