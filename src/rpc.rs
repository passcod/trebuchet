// Why JSON RPC? Simple, lightweight, well-established, can be hand-written in a pinch
// Why Websocket? Duplex, inspectable, trivial to secure, can be used from browsers as-is

pub use self::definer::*;
pub use self::handler::*;

mod definer;
mod handler;
