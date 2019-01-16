use jsonrpc_core::{Error as RpcError, IoHandler, Params, Result as RpcResult, Value};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde_json::from_value;

pub trait RpcDefiner {
    fn rpc(&mut self) -> &mut IoHandler;
    fn init_rpc(&mut self);

    fn define_method<D: DeserializeOwned + std::fmt::Debug + 'static>(
        &mut self,
        name: &'static str,
        imp: fn(D) -> RpcResult<Value>,
    ) {
        self.rpc().add_method(name, move |param| {
            debug!("receiving for typed method {}, parsing payload", name);
            let converted: D = parse_params(param)?;
            info!("handling typed method {}", name);
            imp(converted)
        });
    }
}

pub fn parse_params<D: DeserializeOwned + std::fmt::Debug + 'static>(
    param: Params,
) -> RpcResult<D> {
    debug!("In-> {:?}", param);
    let value = match param {
        Params::None => Value::Array(Vec::new()),
        Params::Array(mut vec) => match vec.len() {
            0 => Value::Array(Vec::new()),
            1 => vec.remove(0),
            _ => Value::Array(vec),
        },
        Params::Map(map) => Value::Object(map),
    };

    debug!("Thru-> {:?}", value);
    from_value::<D>(value)
        .map(|out| {
            debug!("Out-> {:?}", out);
            out
        })
        .map_err(|err| {
            debug!("Err-> expected {}", err);
            RpcError::invalid_params(format!("expected {}", err))
        })
}
