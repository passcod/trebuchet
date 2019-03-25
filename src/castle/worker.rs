use crate::client::Kind;
use crate::inflight::Inflight;
use crate::rpc::RpcHandler;
use crate::Bus;
use jsonrpc_core::{IoHandler, Params, Result as RpcResult};
use log::{debug, info, trace};
use std::thread::{spawn, JoinHandle};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Missive {
    Exit,
    Hello {
        app: String,
        kind: Kind,
        name: String,
        tags: Vec<String>,
    },
}

pub fn worker(ws: ws::Sender, bus: Bus<Missive>) {
    debug!("worker thread start {}", bus.id);

    for missive in bus.iter() {
        trace!("received bus message: {:?}", missive);
        match missive {
            Missive::Exit => {
                bus.send_top(Missive::Exit);
                break;
            }
            _ => {}
        }
    }

    debug!("worker thread end {}", bus.id);
}
