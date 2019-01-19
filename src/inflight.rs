use arc_swap::ArcSwap;
use crossbeam_channel::{bounded, Receiver, Sender};
use jsonrpc_core::{Id, Response};
use log::trace;
use rpds::HashTrieMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct Inflight {
    counter: AtomicUsize,
    store: ArcSwap<HashTrieMap<Id, Sender<Response>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        trace!("incrementic atomic");
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = bounded(1);

        self.store.rcu(|inner| {
            let id = id.clone();
            trace!("rcu over inflight log to insert {:?}", id);
            inner.insert(id, tx.clone())
        });

        (id, rx)
    }

    pub fn recall(&self, id: &Id) -> Option<Sender<Response>> {
        self.store
            .rcu(|inner| {
                let id = id.clone();
                trace!("rcu over inflight log to remove {:?}", id);
                inner.remove(&id)
            })
            .get(&id)
            .map(|tx| {
                trace!("cloning sender");
                tx.clone()
            })
    }
}
