use chashmap::CHashMap;
use crossbeam_channel::{bounded, Receiver, Sender};
use jsonrpc_core::{Id, Response};
use log::trace;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Default)]
pub struct Inflight {
    counter: Arc<AtomicUsize>,
    store: Arc<CHashMap<Id, Sender<Response>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        trace!("incrementic atomic");
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = bounded(0);

        {
            trace!("insert {:?} into inflight log", id);
            self.store.insert(id.clone(), tx);
        }

        (id, rx)
    }

    pub fn recall(&self, id: &Id) -> Option<Sender<Response>> {
        trace!("remove {:?} from inflight log", id);
        self.store.remove(id).map(|tx| {
            trace!("{:?} was in log", id);
            tx
        })
    }
}
