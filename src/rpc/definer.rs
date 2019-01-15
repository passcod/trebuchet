use crate::rpc::FromValue;
use jsonrpc_core::{Error as RpcError, IoHandler, Params, Result as RpcResult, Value};
use log::info;

pub trait RpcDefiner {
    fn rpc(&mut self) -> &mut IoHandler;
    fn init_rpc(&mut self);

    fn define_method<D: FromValue + std::fmt::Debug + 'static>(
        &mut self,
        name: &str,
        imp: fn(D) -> RpcResult<Value>,
    ) {
        self.rpc().add_method(name, move |param| {
            info!("In-> {:?}", param);
            let value = match param {
                Params::None => Value::Array(Vec::new()),
                Params::Array(mut vec) => match vec.len() {
                    0 => Value::Array(Vec::new()),
                    1 => vec.remove(0),
                    _ => Value::Array(vec),
                },
                Params::Map(_) => return Err(RpcError::invalid_params("expected an array")),
                // ^ TODO
            };

            info!("Thru-> {:?}", value);
            let converted = match <D as FromValue>::from(value) {
                Ok(c) => c,
                Err(err) => {
                    info!("Err-> expected {}", err);
                    return Err(RpcError::invalid_params(format!("expected {}", err)));
                }
            };
            info!("Out-> {:?}", converted);
            imp(converted)
        });
    }
}
