use crate::client::Kind;
use crate::db::models::App;
use crate::Bus;
use crossbeam_channel::Sender;
use jsonrpc_core::Result as RpcResult;
use log::trace;

#[derive(Clone, Debug)]
pub enum Missive {
    Exit,
    Hello {
        app: String,
        kind: Kind,
        name: String,
        tags: Vec<String>,
    },
    DataRequest {
        topic: super::data::Topic,
        tx: Sender<RpcResult<Missive>>,
    },
    App(App),
    AppList(Vec<App>),
}

pub fn worker(_ws: ws::Sender, bus: Bus<Missive>) {
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
}
