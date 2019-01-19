// Why JSON RPC? Simple, lightweight, well-established, can be hand-written in a pinch
// Why Websocket? Duplex, inspectable, trivial to secure, can be used from browsers as-is

pub use self::handler::*;
use jsonrpc_core::{Error, ErrorCode, Value};

mod handler;

pub fn app_error(code: i64, message: &str, data: Option<Value>) -> Error {
    Error {
        code: ErrorCode::ServerError(code),
        message: message.into(),
        data,
    }
}
