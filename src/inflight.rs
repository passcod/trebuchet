use arc_swap::ArcSwap;
use futures::sync::oneshot::{channel, Receiver, Sender};
use jsonrpc_core::{Id, Response};
use rpds::HashTrieMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
pub struct Inflight {
    counter: AtomicUsize,
    store: ArcSwap<HashTrieMap<Id, Arc<Sender<Response>>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = channel();

        let atx = Arc::new(tx);
        self.store
            .rcu(|inner| inner.insert(id.clone(), atx.clone()));

        (id, rx)
    }

    pub fn recall(&self, id: &Id) -> Option<Sender<Response>> {
        self.store
            .rcu(|inner| inner.remove(&id.clone()))
            .get(&id) // this should be safe, as we've just removed the only other copy
            .map(|tx| Arc::try_unwrap(tx.clone()).unwrap())
    }
}
